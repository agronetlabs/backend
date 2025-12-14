//! USB HID transport layer for Ledger device communication

use crate::ledger_flex::error::{LedgerError, Result};
use hidapi::{HidApi, HidDevice};

/// Ledger vendor ID
const LEDGER_VENDOR_ID: u16 = 0x2c97;

/// Ledger product IDs
const LEDGER_NANO_S: u16 = 0x0001;
const LEDGER_NANO_X: u16 = 0x0004;
const LEDGER_NANO_S_PLUS: u16 = 0x0005;
const LEDGER_FLEX: u16 = 0x0008;
const LEDGER_STAX: u16 = 0x000a;

/// HID usage page for Ledger
const LEDGER_USAGE_PAGE: u16 = 0xffa0;

/// Communication channel
const LEDGER_CHANNEL: u16 = 0x0101;

/// APDU packet tags
const TAG_APDU: u8 = 0x05;

/// Maximum packet size
const PACKET_SIZE: usize = 64;

/// Timeout for device operations (60 seconds for user confirmation)
const DEVICE_TIMEOUT_MS: i32 = 60_000;

/// Transport layer for Ledger device communication
pub struct LedgerTransport {
    device: HidDevice,
    sequence: u16,
}

impl LedgerTransport {
    /// Connect to a Ledger device
    pub fn connect() -> Result<Self> {
        let api = HidApi::new().map_err(|e| {
            LedgerError::CommunicationError(format!("Failed to initialize HID API: {}", e))
        })?;

        // Try to find any Ledger device
        let device_info = api
            .device_list()
            .find(|dev| {
                dev.vendor_id() == LEDGER_VENDOR_ID
                    && matches!(
                        dev.product_id(),
                        LEDGER_NANO_S | LEDGER_NANO_X | LEDGER_NANO_S_PLUS | LEDGER_FLEX | LEDGER_STAX
                    )
                    && dev.usage_page() == LEDGER_USAGE_PAGE
            })
            .ok_or(LedgerError::DeviceNotFound)?;

        let device = device_info.open_device(&api)?;

        // Set read timeout to 60 seconds for user confirmation
        device
            .set_blocking_mode(true)
            .map_err(|e| LedgerError::CommunicationError(e.to_string()))?;

        Ok(Self {
            device,
            sequence: 0,
        })
    }

    /// Exchange APDU command with the device
    pub fn exchange(&mut self, command: &[u8]) -> Result<Vec<u8>> {
        self.write_apdu(command)?;
        self.read_apdu()
    }

    /// Write APDU command to device
    fn write_apdu(&mut self, command: &[u8]) -> Result<()> {
        let mut packets = Vec::new();
        self.sequence = self.sequence.wrapping_add(1);

        let mut offset = 0;
        let mut seq_idx = 0u16;

        while offset < command.len() {
            let mut packet = vec![0u8; PACKET_SIZE];
            let mut pos = 0;

            // Channel
            packet[pos..pos + 2].copy_from_slice(&LEDGER_CHANNEL.to_be_bytes());
            pos += 2;

            // Tag
            packet[pos] = TAG_APDU;
            pos += 1;

            // Sequence index
            packet[pos..pos + 2].copy_from_slice(&seq_idx.to_be_bytes());
            pos += 2;

            if seq_idx == 0 {
                // First packet includes total length
                let total_len = command.len() as u16;
                packet[pos..pos + 2].copy_from_slice(&total_len.to_be_bytes());
                pos += 2;
            }

            // Copy data
            let remaining = command.len() - offset;
            let available = PACKET_SIZE - pos;
            let to_copy = remaining.min(available);

            packet[pos..pos + to_copy].copy_from_slice(&command[offset..offset + to_copy]);
            offset += to_copy;

            packets.push(packet);
            seq_idx += 1;
        }

        // Write all packets
        for packet in packets {
            self.device
                .write(&packet)
                .map_err(|e| LedgerError::CommunicationError(e.to_string()))?;
        }

        Ok(())
    }

    /// Read APDU response from device
    fn read_apdu(&mut self) -> Result<Vec<u8>> {
        let mut response = Vec::new();
        let mut expected_len = None;
        let mut seq_idx = 0u16;
        let mut packet = vec![0u8; PACKET_SIZE];

        loop {
            // Clear buffer for reuse
            packet.fill(0);
            
            // Read with timeout
            let read_len = self
                .device
                .read_timeout(&mut packet, DEVICE_TIMEOUT_MS)
                .map_err(|e| {
                    if e.to_string().contains("timeout") {
                        LedgerError::Timeout
                    } else {
                        LedgerError::CommunicationError(e.to_string())
                    }
                })?;

            if read_len == 0 {
                return Err(LedgerError::Timeout);
            }

            let mut pos = 0;

            // Verify channel
            let channel = u16::from_be_bytes([packet[pos], packet[pos + 1]]);
            if channel != LEDGER_CHANNEL {
                return Err(LedgerError::InvalidResponse(format!(
                    "Invalid channel: 0x{:04x}",
                    channel
                )));
            }
            pos += 2;

            // Verify tag
            let tag = packet[pos];
            if tag != TAG_APDU {
                return Err(LedgerError::InvalidResponse(format!(
                    "Invalid tag: 0x{:02x}",
                    tag
                )));
            }
            pos += 1;

            // Verify sequence
            let seq = u16::from_be_bytes([packet[pos], packet[pos + 1]]);
            if seq != seq_idx {
                return Err(LedgerError::InvalidResponse(format!(
                    "Invalid sequence: expected {}, got {}",
                    seq_idx, seq
                )));
            }
            pos += 2;

            if seq_idx == 0 {
                // First packet includes total length
                let total_len = u16::from_be_bytes([packet[pos], packet[pos + 1]]) as usize;
                expected_len = Some(total_len);
                pos += 2;
            }

            // Copy data
            let available = read_len - pos;
            let remaining = expected_len.unwrap_or(0) - response.len();
            let to_copy = available.min(remaining);

            response.extend_from_slice(&packet[pos..pos + to_copy]);

            // Check if we have all data
            if let Some(expected) = expected_len {
                if response.len() >= expected {
                    break;
                }
            }

            seq_idx += 1;
        }

        Ok(response)
    }

    /// Disconnect from the device
    pub fn disconnect(self) {
        // HidDevice is automatically closed when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_constants() {
        assert_eq!(LEDGER_VENDOR_ID, 0x2c97);
        assert_eq!(LEDGER_CHANNEL, 0x0101);
        assert_eq!(TAG_APDU, 0x05);
    }
}
