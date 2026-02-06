//! Ethereum Provider Configuration
//! Manages RPC connections to multiple networks

use ethers::providers::{Http, Provider, ProviderError};

#[derive(Clone)]
#[allow(dead_code)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub name: String,
    pub explorer_url: String,
}

/// Get an Ethereum provider for the specified network
pub fn get_provider(network: &str) -> Result<Provider<Http>, ProviderError> {
    let config = get_chain_config(network);
    let provider = Provider::<Http>::try_from(config.rpc_url.as_str())
        .map_err(|e| ProviderError::CustomError(format!("Failed to create provider: {}", e)))?;
    Ok(provider)
}

/// Get chain configuration for a specific network
pub fn get_chain_config(network: &str) -> ChainConfig {
    match network.to_lowercase().as_str() {
        "ethereum" | "mainnet" => ChainConfig {
            rpc_url: std::env::var("ETH_RPC_URL")
                .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string()),
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            explorer_url: "https://etherscan.io".to_string(),
        },
        "sepolia" => ChainConfig {
            rpc_url: std::env::var("SEPOLIA_RPC_URL")
                .unwrap_or_else(|_| "https://eth-sepolia.g.alchemy.com/v2/demo".to_string()),
            chain_id: 11155111,
            name: "Sepolia Testnet".to_string(),
            explorer_url: "https://sepolia.etherscan.io".to_string(),
        },
        "base" | "base-mainnet" => ChainConfig {
            rpc_url: std::env::var("BASE_RPC_URL")
                .unwrap_or_else(|_| "https://mainnet.base.org".to_string()),
            chain_id: 8453,
            name: "Base Mainnet".to_string(),
            explorer_url: "https://basescan.org".to_string(),
        },
        "base-sepolia" => ChainConfig {
            rpc_url: std::env::var("BASE_SEPOLIA_RPC_URL")
                .unwrap_or_else(|_| "https://sepolia.base.org".to_string()),
            chain_id: 84532,
            name: "Base Sepolia".to_string(),
            explorer_url: "https://sepolia.basescan.org".to_string(),
        },
        "arbitrum" | "arbitrum-mainnet" => ChainConfig {
            rpc_url: std::env::var("ARBITRUM_RPC_URL")
                .unwrap_or_else(|_| "https://arb1.arbitrum.io/rpc".to_string()),
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            explorer_url: "https://arbiscan.io".to_string(),
        },
        "arbitrum-sepolia" => ChainConfig {
            rpc_url: std::env::var("ARBITRUM_SEPOLIA_RPC_URL")
                .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string()),
            chain_id: 421614,
            name: "Arbitrum Sepolia".to_string(),
            explorer_url: "https://sepolia.arbiscan.io".to_string(),
        },
        _ => ChainConfig {
            rpc_url: std::env::var("ETH_RPC_URL")
                .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string()),
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            explorer_url: "https://etherscan.io".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_configs() {
        let eth_config = get_chain_config("ethereum");
        assert_eq!(eth_config.chain_id, 1);
        assert_eq!(eth_config.name, "Ethereum Mainnet");

        let sepolia_config = get_chain_config("sepolia");
        assert_eq!(sepolia_config.chain_id, 11155111);

        let base_config = get_chain_config("base");
        assert_eq!(base_config.chain_id, 8453);
    }
}
