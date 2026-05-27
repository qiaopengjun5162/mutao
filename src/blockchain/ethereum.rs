use async_trait::async_trait;

use super::{BlockchainError, ChainAdapter, ChainRecord, ChainType, SwapProof, TransactionStatus};

pub struct EthereumConfig {
    pub rpc_url: String,
    pub contract_address: String,
    pub chain_id: u64,
}

pub struct EthereumAdapter {
    #[allow(dead_code)]
    config: EthereumConfig,
}

impl EthereumAdapter {
    pub fn new(config: EthereumConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ChainAdapter for EthereumAdapter {
    fn chain_type(&self) -> ChainType {
        ChainType::Ethereum
    }

    fn chain_name(&self) -> &str {
        "Ethereum"
    }

    async fn record_swap(&self, proof: &SwapProof) -> Result<ChainRecord, BlockchainError> {
        tracing::info!(
            "以太坊存证: item={}, from={}, to={}",
            proof.item_id,
            proof.from_user,
            proof.to_user
        );

        // TODO: 构造 calldata -> 签名 -> 发送 RPC -> 等待确认

        Ok(ChainRecord {
            chain: ChainType::Ethereum,
            tx_hash: format!("0x{:064x}", 0),
            block_number: 0,
            timestamp: chrono::Utc::now().timestamp() as u64,
            status: TransactionStatus::Pending,
            data: serde_json::to_value(proof).unwrap_or_default(),
        })
    }

    async fn get_history(&self, item_id: &str) -> Result<Vec<ChainRecord>, BlockchainError> {
        tracing::info!("查询以太坊历史: item={}", item_id);
        Ok(vec![])
    }

    async fn get_swap_count(&self, item_id: &str) -> Result<u64, BlockchainError> {
        tracing::info!("查询以太坊交换次数: item={}", item_id);
        Ok(0)
    }

    async fn verify_transaction(&self, tx_hash: &str) -> Result<bool, BlockchainError> {
        tracing::info!("验证以太坊交易: tx={}", tx_hash);
        Ok(false)
    }
}
