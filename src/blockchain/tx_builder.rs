//! Transaction Builder
//! Utilities for building, estimating gas, and managing Ethereum transactions

use ethers::prelude::*;
use ethers::types::{transaction::eip2718::TypedTransaction, TransactionRequest};
use std::sync::Arc;

/// Transaction builder for EIP-1559 transactions
#[allow(dead_code)]
pub struct TxBuilder<M: Middleware> {
    client: Arc<M>,
    gas_price_multiplier: f64,
    max_gas_price_gwei: u64,
}

#[allow(dead_code)]
impl<M: Middleware> TxBuilder<M> {
    /// Create a new transaction builder
    pub fn new(client: Arc<M>) -> Self {
        let gas_price_multiplier = std::env::var("GAS_PRICE_MULTIPLIER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.1);

        let max_gas_price_gwei = std::env::var("MAX_GAS_PRICE_GWEI")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Self {
            client,
            gas_price_multiplier,
            max_gas_price_gwei,
        }
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
    ) -> Result<U256, Box<dyn std::error::Error>>
    where
        M::Error: 'static,
    {
        let gas_estimate = self.client.estimate_gas(tx, None).await?;
        // Add configurable buffer to gas estimate (default 20%)
        let buffer_percent = std::env::var("GAS_ESTIMATE_BUFFER_PERCENT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(20);
        let gas_with_buffer = gas_estimate * (100 + buffer_percent) / 100;
        Ok(gas_with_buffer)
    }

    /// Get current gas price with multiplier applied
    pub async fn get_gas_price(&self) -> Result<U256, Box<dyn std::error::Error>>
    where
        M::Error: 'static,
    {
        let gas_price = self.client.get_gas_price().await?;
        let multiplied = (gas_price.as_u128() as f64 * self.gas_price_multiplier) as u128;
        let result = U256::from(multiplied);

        // Check against max gas price
        let max_price = U256::from(self.max_gas_price_gwei) * U256::exp10(9);
        if result > max_price {
            Ok(max_price)
        } else {
            Ok(result)
        }
    }

    /// Get the next nonce for an address
    pub async fn get_nonce(&self, address: Address) -> Result<U256, Box<dyn std::error::Error>>
    where
        M::Error: 'static,
    {
        let nonce = self
            .client
            .get_transaction_count(address, None)
            .await?;
        Ok(nonce)
    }

    /// Build a transaction with automatic gas estimation
    pub async fn build_tx(
        &self,
        from: Address,
        to: Address,
        value: U256,
        data: Option<Bytes>,
    ) -> Result<TransactionRequest, Box<dyn std::error::Error>>
    where
        M::Error: 'static,
    {
        let nonce = self.get_nonce(from).await?;
        let gas_price = self.get_gas_price().await?;

        let mut tx = TransactionRequest::new()
            .from(from)
            .to(to)
            .value(value)
            .nonce(nonce)
            .gas_price(gas_price);

        if let Some(data) = data {
            tx = tx.data(data);
        }

        // Estimate gas
        let typed_tx: TypedTransaction = tx.clone().into();
        let gas_limit = self.estimate_gas(&typed_tx).await?;
        tx = tx.gas(gas_limit);

        Ok(tx)
    }

    /// Send transaction with retry logic
    pub async fn send_transaction_with_retry<T: Into<TypedTransaction>>(
        &self,
        tx: T,
        max_retries: u32,
    ) -> Result<PendingTransaction<'_, M::Provider>, Box<dyn std::error::Error>>
    where
        M::Error: 'static,
    {
        let mut retries = 0;
        let tx: TypedTransaction = tx.into();

        loop {
            match self.client.send_transaction(tx.clone(), None).await {
                Ok(pending_tx) => return Ok(pending_tx),
                Err(e) => {
                    if retries >= max_retries {
                        return Err(Box::new(e));
                    }
                    retries += 1;
                    tracing::warn!(
                        "Transaction failed (attempt {}/{}): {}",
                        retries,
                        max_retries,
                        e
                    );
                    // Exponential backoff with max 30 seconds cap
                    let delay = std::cmp::min(2_u64.pow(retries), 30);
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_builder_config() {
        // Don't mutate env in tests (it is process-global and can be unsafe in recent Rust).
        let provider = ethers::providers::Provider::<ethers::providers::Http>::try_from(
            "http://localhost:8545",
        )
        .expect("provider creation should succeed with dummy URL");

        let builder = TxBuilder::new(Arc::new(provider));
        assert!(builder.gas_price_multiplier > 0.0);
        assert!(builder.max_gas_price_gwei > 0);
    }
}
