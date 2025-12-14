# Ledger Flex Hardware Wallet Integration

This document describes the Ledger Flex hardware wallet integration for secure transaction signing in the AgroNet backend.

## Overview

The Ledger Flex integration provides hardware-based transaction signing where private keys never leave the device, ensuring maximum security for production environments.

## Supported Devices

- Ledger Nano S
- Ledger Nano X
- Ledger Nano S Plus
- Ledger Flex
- Ledger Stax

## Configuration

Add the following environment variables to your `.env` file:

```bash
# Enable or disable Ledger signing
LEDGER_ENABLED=false

# BIP32 derivation path for Ethereum
LEDGER_DERIVATION_PATH="m/44'/60'/0'/0/0"

# Require user confirmation on device
LEDGER_REQUIRE_CONFIRMATION=true
```

### Configuration Options

- **LEDGER_ENABLED**: Set to `true` to enable hardware wallet signing, `false` to use mock signing (default: `false`)
- **LEDGER_DERIVATION_PATH**: BIP32 path for key derivation (default: `"m/44'/60'/0'/0/0"` - first Ethereum account)
- **LEDGER_REQUIRE_CONFIRMATION**: Always require user confirmation on device (default: `true`)

## Usage

### Automatic Integration

When `LEDGER_ENABLED=true`, the Ethereum settlement endpoint (`/api/onchain/settle/ethereum`) will automatically use the Ledger device for signing transactions.

### Manual Usage

```rust
use agronet_backend::ledger_flex::{LedgerTransport, LedgerSigner};

// Connect to Ledger device
let transport = LedgerTransport::connect()?;
let mut signer = LedgerSigner::new(transport);

// Get Ethereum address from device
let address = signer.get_address("m/44'/60'/0'/0/0")?;
println!("Address: {}", address);

// Sign a transaction
let tx_data = vec![/* RLP-encoded transaction */];
let signature = signer.sign_transaction(&tx_data, "m/44'/60'/0'/0/0")?;
```

## Security Features

1. **Hardware Security**: Private keys never leave the Ledger device
2. **User Confirmation**: All transactions require physical confirmation on the device
3. **Timeout Protection**: 60-second timeout for user confirmation to prevent hanging
4. **Multi-device Support**: Automatically detects and connects to available Ledger devices
5. **Fallback Mechanism**: If Ledger signing fails, falls back to mock signing in development

## System Requirements

### Linux

Install the required system dependencies:

```bash
sudo apt-get update
sudo apt-get install -y libudev-dev libusb-1.0-0-dev
```

### macOS

No additional dependencies required - hidapi uses IOKit framework.

### Windows

Requires Windows SDK for HID API support.

## Troubleshooting

### Device Not Found

**Error**: `Ledger device not found or not connected`

**Solutions**:
1. Ensure the Ledger device is connected via USB
2. Check that the device is unlocked
3. Verify that the Ethereum app is open on the device
4. On Linux, check USB permissions

### User Rejected

**Error**: `User rejected the transaction on device`

**Solutions**:
1. Check the transaction details on the device screen
2. Approve the transaction by pressing the right button
3. Ensure you're not accidentally pressing the left button (reject)

### Timeout

**Error**: `Timeout waiting for user confirmation`

**Solutions**:
1. Respond faster to the device prompt (60-second limit)
2. Check that the device is not frozen
3. Restart the device if unresponsive

### App Not Open

**Error**: `Ethereum app not open on Ledger device`

**Solutions**:
1. Navigate to the Ethereum app on your Ledger device
2. Open the Ethereum app
3. Ensure the app is fully loaded before making requests

## Development

### Building

```bash
# Install system dependencies (Linux)
sudo apt-get install -y libudev-dev libusb-1.0-0-dev

# Build the project
SQLX_OFFLINE=true cargo build
```

### Testing

```bash
# Run unit tests
SQLX_OFFLINE=true cargo test --bin AgroNet_backend

# Run only Ledger Flex tests
SQLX_OFFLINE=true cargo test ledger_flex
```

### Testing Without Hardware

Set `LEDGER_ENABLED=false` in your `.env` file to use mock signing for development and testing without requiring a physical Ledger device.

## API Response

When Ledger signing is enabled, the settlement response will include:

```json
{
  "tx_hash": "0x...",
  "network": "ethereum",
  "status": "signed",
  "validation_status": "approved",
  "validation_reason": "OK"
}
```

Note the `status` field:
- `"signed"`: Transaction was signed with Ledger hardware wallet
- `"mocked"`: Transaction used mock signing (development mode)

## Security Best Practices

1. **Always verify addresses**: Double-check the address shown on the Ledger screen
2. **Use hardware in production**: Enable Ledger signing for all production deployments
3. **Secure the device**: Keep your Ledger device in a secure location
4. **Backup recovery phrase**: Store your 24-word recovery phrase in a secure, offline location
5. **Keep firmware updated**: Regularly update your Ledger device firmware
6. **Test derivation paths**: Verify the derivation path produces the expected addresses before use

## License

This integration uses the `hidapi` crate (MIT/Apache-2.0 licensed) for USB HID communication.
