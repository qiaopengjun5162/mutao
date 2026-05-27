use async_trait::async_trait;

use super::{BlockchainError, ChainAdapter, ChainRecord, ChainType, SwapProof, TransactionStatus};

pub struct SolanaConfig {
    pub rpc_url: String,
    pub program_id: String,
}

pub struct SolanaAdapter {
    #[allow(dead_code)]
    config: SolanaConfig,
}

impl SolanaAdapter {
    pub fn new(config: SolanaConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ChainAdapter for SolanaAdapter {
    fn chain_type(&self) -> ChainType {
        ChainType::Solana
    }

    fn chain_name(&self) -> &str {
        "Solana"
    }

    async fn record_swap(&self, proof: &SwapProof) -> Result<ChainRecord, BlockchainError> {
        tracing::info!(
            "Solana 存证: item={}, from={}, to={}",
            proof.item_id,
            proof.from_user,
            proof.to_user
        );

        // TODO: 构造 instruction -> Transaction -> 签名发送 -> 确认

        Ok(ChainRecord {
            chain: ChainType::Solana,
            tx_hash: String::new(),
            block_number: 0,
            timestamp: chrono::Utc::now().timestamp() as u64,
            status: TransactionStatus::Pending,
            data: serde_json::to_value(proof).unwrap_or_default(),
        })
    }

    async fn get_history(&self, item_id: &str) -> Result<Vec<ChainRecord>, BlockchainError> {
        tracing::info!("查询 Solana 历史: item={}", item_id);
        Ok(vec![])
    }

    async fn get_swap_count(&self, item_id: &str) -> Result<u64, BlockchainError> {
        tracing::info!("查询 Solana 交换次数: item={}", item_id);
        Ok(0)
    }

    async fn verify_transaction(&self, tx_hash: &str) -> Result<bool, BlockchainError> {
        tracing::info!("验证 Solana 交易: tx={}", tx_hash);
        Ok(false)
    }
}
