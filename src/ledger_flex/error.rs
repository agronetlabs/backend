//! Error types for Ledger Flex hardware wallet integration

use std::fmt;

#[derive(Debug, Clone)]
pub enum LedgerError {
    /// Ledger device not found or not connected
    DeviceNotFound,
    /// User rejected the transaction on the device
    UserRejected,
    /// Ethereum app not open on the device
    AppNotOpen,
    /// Invalid response from device
    InvalidResponse(String),
    /// Communication error with device
    CommunicationError(String),
    /// Invalid derivation path
    InvalidPath(String),
    /// Timeout waiting for user confirmation
    Timeout,
    /// Device is locked
    DeviceLocked,
    /// Invalid APDU response status code
    InvalidStatusCode(u16),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::DeviceNotFound => write!(f, "Ledger device not found or not connected"),
            LedgerError::UserRejected => write!(f, "User rejected the transaction on device"),
            LedgerError::AppNotOpen => write!(f, "Ethereum app not open on Ledger device"),
            LedgerError::InvalidResponse(msg) => write!(f, "Invalid response from device: {}", msg),
            LedgerError::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            LedgerError::InvalidPath(msg) => write!(f, "Invalid derivation path: {}", msg),
            LedgerError::Timeout => write!(f, "Timeout waiting for user confirmation"),
            LedgerError::DeviceLocked => write!(f, "Device is locked, please unlock it"),
            LedgerError::InvalidStatusCode(code) => write!(f, "Invalid status code: 0x{:04x}", code),
        }
    }
}

impl std::error::Error for LedgerError {}

impl From<hidapi::HidError> for LedgerError {
    fn from(err: hidapi::HidError) -> Self {
        LedgerError::CommunicationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, LedgerError>;
