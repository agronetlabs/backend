//! Ledger signer for secure transaction signing

use crate::ledger_flex::commands::{
    get_public_key_command, parse_public_key_response, ApduCommand, ApduResponse,
    INS_SIGN_MESSAGE, INS_SIGN_TRANSACTION,
};
use crate::ledger_flex::error::{LedgerError, Result};
use crate::ledger_flex::transport::LedgerTransport;

/// Ledger signer for Ethereum transactions
pub struct LedgerSigner {
    transport: LedgerTransport,
}

impl LedgerSigner {
    /// Create a new Ledger signer
    pub fn new(transport: LedgerTransport) -> Self {
        Self { transport }
    }

    /// Get public key from Ledger device
    /// 
    /// # Arguments
    /// * `path` - BIP32 derivation path (e.g., "m/44'/60'/0'/0/0")
    /// 
    /// # Returns
    /// * Uncompressed public key (65 bytes)
    pub fn get_public_key(&mut self, path: &str) -> Result<Vec<u8>> {
        let command = get_public_key_command(path, false)?;
        let response_bytes = self.transport.exchange(&command.to_bytes())?;
        let response = ApduResponse::from_bytes(&response_bytes)?;
        
        let (pubkey, _address) = parse_public_key_response(&response)?;
        Ok(pubkey)
    }

    /// Get Ethereum address from Ledger device
    /// 
    /// # Arguments
    /// * `path` - BIP32 derivation path (e.g., "m/44'/60'/0'/0/0")
    /// 
    /// # Returns
    /// * Ethereum address as hex string (with 0x prefix)
    pub fn get_address(&mut self, path: &str) -> Result<String> {
        let command = get_public_key_command(path, false)?;
        let response_bytes = self.transport.exchange(&command.to_bytes())?;
        let response = ApduResponse::from_bytes(&response_bytes)?;
        
        let (_pubkey, address_bytes) = parse_public_key_response(&response)?;
        
        // Address is returned as ASCII hex string
        let address_str = String::from_utf8(address_bytes)
            .map_err(|e| LedgerError::InvalidResponse(format!("Invalid address encoding: {}", e)))?;
        
        // Ensure it has 0x prefix
        if address_str.starts_with("0x") {
            Ok(address_str)
        } else {
            Ok(format!("0x{}", address_str))
        }
    }

    /// Sign an Ethereum transaction
    /// 
    /// # Arguments
    /// * `tx_data` - RLP-encoded transaction data
    /// * `path` - BIP32 derivation path (e.g., "m/44'/60'/0'/0/0")
    /// 
    /// # Returns
    /// * Signature (v, r, s) as bytes
    pub fn sign_transaction(&mut self, tx_data: &[u8], path: &str) -> Result<Vec<u8>> {
        let path_data = crate::ledger_flex::commands::parse_derivation_path(path)?;
        
        // Prepare transaction data with path
        let mut data = path_data.clone();
        data.extend_from_slice(tx_data);

        // For large transactions, we need to send in chunks
        let chunks = chunk_data(&data, 150); // Max ~150 bytes per chunk for safety
        
        for (i, chunk) in chunks.iter().enumerate() {
            let p1 = if i == 0 { 0x00 } else { 0x80 }; // First chunk vs continuation
            let p2 = 0x00;
            
            let command = ApduCommand::new(INS_SIGN_TRANSACTION, p1, p2, chunk.to_vec());
            let response_bytes = self.transport.exchange(&command.to_bytes())?;
            let response = ApduResponse::from_bytes(&response_bytes)?;
            
            // Only the last chunk should return the signature
            if i == chunks.len() - 1 {
                response.check_status()?;
                return Ok(response.data);
            } else {
                // Intermediate chunks should return OK status
                response.check_status()?;
            }
        }

        Err(LedgerError::InvalidResponse(
            "No signature returned".to_string(),
        ))
    }

    /// Sign an arbitrary message (EIP-191 personal sign)
    /// 
    /// # Arguments
    /// * `message` - Message to sign
    /// * `path` - BIP32 derivation path (e.g., "m/44'/60'/0'/0/0")
    /// 
    /// # Returns
    /// * Signature (v, r, s) as bytes
    pub fn sign_message(&mut self, message: &[u8], path: &str) -> Result<Vec<u8>> {
        let path_data = crate::ledger_flex::commands::parse_derivation_path(path)?;
        
        // Prepare message with length prefix
        let mut data = path_data.clone();
        let msg_len = message.len() as u32;
        data.extend_from_slice(&msg_len.to_be_bytes());
        data.extend_from_slice(message);

        // Send in chunks if necessary
        let chunks = chunk_data(&data, 150);
        
        for (i, chunk) in chunks.iter().enumerate() {
            let p1 = if i == 0 { 0x00 } else { 0x80 };
            let p2 = 0x00;
            
            let command = ApduCommand::new(INS_SIGN_MESSAGE, p1, p2, chunk.to_vec());
            let response_bytes = self.transport.exchange(&command.to_bytes())?;
            let response = ApduResponse::from_bytes(&response_bytes)?;
            
            if i == chunks.len() - 1 {
                response.check_status()?;
                return Ok(response.data);
            } else {
                response.check_status()?;
            }
        }

        Err(LedgerError::InvalidResponse(
            "No signature returned".to_string(),
        ))
    }

    /// Close connection to the device
    pub fn disconnect(self) {
        self.transport.disconnect();
    }
}

/// Split data into chunks
fn chunk_data(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let end = (offset + chunk_size).min(data.len());
        chunks.push(data[offset..end].to_vec());
        offset = end;
    }

    chunks
}

/// Parse signature from response
/// Returns (v, r, s)
pub fn parse_signature(sig_data: &[u8]) -> Result<(u8, [u8; 32], [u8; 32])> {
    if sig_data.len() != 65 {
        return Err(LedgerError::InvalidResponse(format!(
            "Invalid signature length: expected 65, got {}",
            sig_data.len()
        )));
    }

    let v = sig_data[0];
    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    
    r.copy_from_slice(&sig_data[1..33]);
    s.copy_from_slice(&sig_data[33..65]);

    Ok((v, r, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_data() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let chunks = chunk_data(&data, 3);
        
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], vec![1, 2, 3]);
        assert_eq!(chunks[1], vec![4, 5, 6]);
        assert_eq!(chunks[2], vec![7, 8, 9]);
        assert_eq!(chunks[3], vec![10]);
    }

    #[test]
    fn test_parse_signature() {
        let mut sig_data = vec![0u8; 65];
        sig_data[0] = 27; // v
        sig_data[1] = 0xff; // first byte of r
        sig_data[33] = 0xaa; // first byte of s
        
        let (v, r, s) = parse_signature(&sig_data).unwrap();
        
        assert_eq!(v, 27);
        assert_eq!(r[0], 0xff);
        assert_eq!(s[0], 0xaa);
    }
}
