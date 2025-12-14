//! Ledger Flex Hardware Wallet Integration
//! 
//! Secure transaction signing for settlement operations using Ledger hardware wallets.
//! This module provides secure, hardware-based transaction signing where private keys
//! never leave the device.
//! 
//! # Security Features
//! - Private keys never leave the device
//! - User confirmation required on device for all signatures
//! - 60-second timeout for user confirmation
//! - Support for multiple Ledger devices (Nano S, Nano X, Nano S Plus, Flex, Stax)
//! 
//! # Example Usage
//! ```rust,no_run
//! use agronet_backend::ledger_flex::{LedgerTransport, LedgerSigner};
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to Ledger
//! let transport = LedgerTransport::connect()?;
//! let mut signer = LedgerSigner::new(transport);
//! 
//! // Get Ethereum address
//! let address = signer.get_address("m/44'/60'/0'/0/0")?;
//! println!("Address: {}", address);
//! 
//! // Sign a transaction
//! let tx_data = vec![/* RLP-encoded transaction */];
//! let signature = signer.sign_transaction(&tx_data, "m/44'/60'/0'/0/0")?;
//! # Ok(())
//! # }
//! ```

pub mod commands;
pub mod error;
pub mod signer;
pub mod transport;

// Re-export main types for convenience
pub use signer::LedgerSigner;
pub use transport::LedgerTransport;
