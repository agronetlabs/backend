//! APDU commands for Ethereum app on Ledger devices
//! 
//! APDU (Application Protocol Data Unit) is the communication protocol
//! used to send commands to smart card applications like Ledger.

use crate::ledger_flex::error::{LedgerError, Result};

/// Ethereum app APDU instruction codes
pub const INS_GET_PUBLIC_KEY: u8 = 0x02;
pub const INS_SIGN_TRANSACTION: u8 = 0x04;
pub const INS_SIGN_MESSAGE: u8 = 0x08;
pub const INS_GET_APP_CONFIGURATION: u8 = 0x06;

/// APDU class byte for Ethereum app
pub const CLA: u8 = 0xe0;

/// APDU success status code
pub const SW_OK: u16 = 0x9000;

/// APDU status codes
pub const SW_USER_REJECTED: u16 = 0x6985;
pub const SW_DEVICE_LOCKED: u16 = 0x6982;
pub const SW_INCORRECT_DATA: u16 = 0x6a80;
pub const SW_CONDITIONS_NOT_SATISFIED: u16 = 0x6985;

/// Maximum APDU payload size
pub const MAX_APDU_SIZE: usize = 255;

/// APDU command structure
#[derive(Debug, Clone)]
pub struct ApduCommand {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
}

impl ApduCommand {
    /// Create a new APDU command
    pub fn new(ins: u8, p1: u8, p2: u8, data: Vec<u8>) -> Self {
        Self {
            cla: CLA,
            ins,
            p1,
            p2,
            data,
        }
    }

    /// Serialize APDU command to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.cla);
        bytes.push(self.ins);
        bytes.push(self.p1);
        bytes.push(self.p2);
        bytes.push(self.data.len() as u8);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// APDU response structure
#[derive(Debug, Clone)]
pub struct ApduResponse {
    pub data: Vec<u8>,
    pub status: u16,
}

impl ApduResponse {
    /// Parse response from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 {
            return Err(LedgerError::InvalidResponse(
                "Response too short".to_string(),
            ));
        }

        let data_len = bytes.len() - 2;
        let data = bytes[..data_len].to_vec();
        let status = u16::from_be_bytes([bytes[data_len], bytes[data_len + 1]]);

        Ok(Self { data, status })
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.status == SW_OK
    }

    /// Convert response to error if not successful
    pub fn check_status(&self) -> Result<()> {
        match self.status {
            SW_OK => Ok(()),
            SW_USER_REJECTED => Err(LedgerError::UserRejected),
            SW_DEVICE_LOCKED => Err(LedgerError::DeviceLocked),
            SW_INCORRECT_DATA => Err(LedgerError::InvalidResponse(
                "Incorrect data sent to device".to_string(),
            )),
            _ => Err(LedgerError::InvalidStatusCode(self.status)),
        }
    }
}

/// Parse BIP32 derivation path to bytes
/// Format: "m/44'/60'/0'/0/0" -> [44, 60, 0, 0, 0] with hardened flags
pub fn parse_derivation_path(path: &str) -> Result<Vec<u8>> {
    let path = path.trim_start_matches("m/");
    let components: Vec<&str> = path.split('/').collect();

    if components.is_empty() || components.len() > 10 {
        return Err(LedgerError::InvalidPath(
            "Path must have 1-10 components".to_string(),
        ));
    }

    let mut result = Vec::new();
    result.push(components.len() as u8);

    for component in components {
        let (num_str, hardened) = if component.ends_with('\'') {
            (&component[..component.len() - 1], true)
        } else {
            (component, false)
        };

        let num: u32 = num_str
            .parse()
            .map_err(|_| LedgerError::InvalidPath(format!("Invalid number: {}", num_str)))?;

        let value = if hardened { num | 0x80000000 } else { num };
        result.extend_from_slice(&value.to_be_bytes());
    }

    Ok(result)
}

/// Build GET_PUBLIC_KEY command
pub fn get_public_key_command(derivation_path: &str, display: bool) -> Result<ApduCommand> {
    let path_data = parse_derivation_path(derivation_path)?;
    let p1 = if display { 0x01 } else { 0x00 };
    let p2 = 0x00; // No chain code

    Ok(ApduCommand::new(INS_GET_PUBLIC_KEY, p1, p2, path_data))
}

/// Parse public key response
pub fn parse_public_key_response(response: &ApduResponse) -> Result<(Vec<u8>, Vec<u8>)> {
    response.check_status()?;

    if response.data.len() < 66 {
        return Err(LedgerError::InvalidResponse(
            "Public key response too short".to_string(),
        ));
    }

    let pubkey_len = response.data[0] as usize;
    if pubkey_len != 65 {
        return Err(LedgerError::InvalidResponse(format!(
            "Invalid public key length: {}",
            pubkey_len
        )));
    }

    let pubkey = response.data[1..66].to_vec();
    let address_len = response.data[66] as usize;

    if response.data.len() < 67 + address_len {
        return Err(LedgerError::InvalidResponse(
            "Address data missing".to_string(),
        ));
    }

    let address = response.data[67..67 + address_len].to_vec();

    Ok((pubkey, address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_derivation_path() {
        let path = "m/44'/60'/0'/0/0";
        let result = parse_derivation_path(path).unwrap();
        
        // Should have: 1 byte for count + 5 * 4 bytes for components = 21 bytes
        assert_eq!(result.len(), 21);
        assert_eq!(result[0], 5); // 5 components
    }

    #[test]
    fn test_apdu_command_serialization() {
        let cmd = ApduCommand::new(INS_GET_PUBLIC_KEY, 0x00, 0x00, vec![0x01, 0x02]);
        let bytes = cmd.to_bytes();
        
        assert_eq!(bytes[0], CLA);
        assert_eq!(bytes[1], INS_GET_PUBLIC_KEY);
        assert_eq!(bytes[2], 0x00);
        assert_eq!(bytes[3], 0x00);
        assert_eq!(bytes[4], 2); // data length
        assert_eq!(bytes[5], 0x01);
        assert_eq!(bytes[6], 0x02);
    }

    #[test]
    fn test_apdu_response_parsing() {
        let bytes = vec![0x01, 0x02, 0x03, 0x90, 0x00];
        let response = ApduResponse::from_bytes(&bytes).unwrap();
        
        assert_eq!(response.data, vec![0x01, 0x02, 0x03]);
        assert_eq!(response.status, SW_OK);
        assert!(response.is_success());
    }
}
