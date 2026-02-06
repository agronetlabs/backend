//! Network Configuration
//! Defines supported Ethereum networks and their configurations

use ethers::types::Address;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    EthereumMainnet,
    EthereumSepolia,
    BaseMainnet,
    BaseSepolia,
    ArbitrumMainnet,
    ArbitrumSepolia,
}

impl Network {
    /// Get the RPC URL for this network
    #[allow(dead_code)]
    pub fn rpc_url(&self) -> String {
        match self {
            Network::EthereumMainnet => {
                std::env::var("ETH_RPC_URL")
                    .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string())
            }
            Network::EthereumSepolia => {
                std::env::var("SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://eth-sepolia.g.alchemy.com/v2/demo".to_string())
            }
            Network::BaseMainnet => {
                std::env::var("BASE_RPC_URL")
                    .unwrap_or_else(|_| "https://mainnet.base.org".to_string())
            }
            Network::BaseSepolia => {
                std::env::var("BASE_SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://sepolia.base.org".to_string())
            }
            Network::ArbitrumMainnet => {
                std::env::var("ARBITRUM_RPC_URL")
                    .unwrap_or_else(|_| "https://arb1.arbitrum.io/rpc".to_string())
            }
            Network::ArbitrumSepolia => {
                std::env::var("ARBITRUM_SEPOLIA_RPC_URL")
                    .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string())
            }
        }
    }

    /// Get the chain ID for this network
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::EthereumMainnet => 1,
            Network::EthereumSepolia => 11155111,
            Network::BaseMainnet => 8453,
            Network::BaseSepolia => 84532,
            Network::ArbitrumMainnet => 42161,
            Network::ArbitrumSepolia => 421614,
        }
    }

    /// Get the block explorer URL for this network
    pub fn explorer(&self) -> &str {
        match self {
            Network::EthereumMainnet => "https://etherscan.io",
            Network::EthereumSepolia => "https://sepolia.etherscan.io",
            Network::BaseMainnet => "https://basescan.org",
            Network::BaseSepolia => "https://sepolia.basescan.org",
            Network::ArbitrumMainnet => "https://arbiscan.io",
            Network::ArbitrumSepolia => "https://sepolia.arbiscan.io",
        }
    }

    /// Get the USDT contract address for this network
    pub fn usdt_address(&self) -> Address {
        match self {
            Network::EthereumMainnet => {
                Address::from_str(
                    &std::env::var("USDT_CONTRACT")
                        .unwrap_or_else(|_| "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::EthereumSepolia => {
                Address::from_str(
                    &std::env::var("SEPOLIA_USDT_CONTRACT")
                        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::BaseMainnet => {
                Address::from_str(
                    &std::env::var("BASE_USDT_CONTRACT")
                        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::BaseSepolia => {
                Address::from_str(
                    &std::env::var("BASE_SEPOLIA_USDT_CONTRACT")
                        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::ArbitrumMainnet => {
                Address::from_str(
                    &std::env::var("ARBITRUM_USDT_CONTRACT")
                        .unwrap_or_else(|_| "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::ArbitrumSepolia => {
                Address::from_str(
                    &std::env::var("ARBITRUM_SEPOLIA_USDT_CONTRACT")
                        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
        }
    }

    /// Get the USDC contract address for this network
    pub fn usdc_address(&self) -> Address {
        match self {
            Network::EthereumMainnet => {
                Address::from_str(
                    &std::env::var("USDC_CONTRACT")
                        .unwrap_or_else(|_| "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::EthereumSepolia => {
                Address::from_str(
                    &std::env::var("SEPOLIA_USDC_CONTRACT")
                        .unwrap_or_else(|_| "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::BaseMainnet => {
                Address::from_str(
                    &std::env::var("BASE_USDC_CONTRACT")
                        .unwrap_or_else(|_| "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::BaseSepolia => {
                Address::from_str(
                    &std::env::var("BASE_SEPOLIA_USDC_CONTRACT")
                        .unwrap_or_else(|_| "0x036CbD53842c5426634e7929541eC2318f3dCF7e".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::ArbitrumMainnet => {
                Address::from_str(
                    &std::env::var("ARBITRUM_USDC_CONTRACT")
                        .unwrap_or_else(|_| "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
            Network::ArbitrumSepolia => {
                Address::from_str(
                    &std::env::var("ARBITRUM_SEPOLIA_USDC_CONTRACT")
                        .unwrap_or_else(|_| "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d".to_string())
                ).unwrap_or_else(|_| Address::zero())
            }
        }
    }

    /// Get ERC8040 contract address for this network
    #[allow(dead_code)]
    pub fn erc8040_address(&self) -> Address {
        Address::from_str(
            &std::env::var("ERC8040_CONTRACT")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
        ).unwrap_or_else(|_| Address::zero())
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "mainnet" | "ethereum-mainnet" => Ok(Network::EthereumMainnet),
            "sepolia" | "ethereum-sepolia" => Ok(Network::EthereumSepolia),
            "base" | "base-mainnet" => Ok(Network::BaseMainnet),
            "base-sepolia" => Ok(Network::BaseSepolia),
            "arbitrum" | "arbitrum-mainnet" => Ok(Network::ArbitrumMainnet),
            "arbitrum-sepolia" => Ok(Network::ArbitrumSepolia),
            _ => Err(format!("Unknown network: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_chain_ids() {
        assert_eq!(Network::EthereumMainnet.chain_id(), 1);
        assert_eq!(Network::EthereumSepolia.chain_id(), 11155111);
        assert_eq!(Network::BaseMainnet.chain_id(), 8453);
        assert_eq!(Network::ArbitrumMainnet.chain_id(), 42161);
    }

    #[test]
    fn test_network_from_str() {
        assert_eq!(Network::from_str("ethereum").unwrap(), Network::EthereumMainnet);
        assert_eq!(Network::from_str("sepolia").unwrap(), Network::EthereumSepolia);
        assert_eq!(Network::from_str("base").unwrap(), Network::BaseMainnet);
        assert!(Network::from_str("unknown").is_err());
    }
}
