pub mod ethereum;
pub mod solana;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("RPC 调用失败: {0}")]
    RpcError(String),

    #[error("交易失败: {0}")]
    TransactionError(String),

    #[error("合约调用失败: {0}")]
    ContractError(String),

    #[error("签名失败: {0}")]
    SigningError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("不支持的链: {0}")]
    UnsupportedChain(String),
}

/// 链类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChainType {
    Ethereum,
    Solana,
    Move,
}

/// 交易状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed(String),
}

/// 链上交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRecord {
    pub chain: ChainType,
    pub tx_hash: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub data: serde_json::Value,
}

/// 存证数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapProof {
    pub item_id: String,
    pub from_user: String,
    pub to_user: String,
    pub item_name: String,
    pub message: String,
    pub swap_count: u64,
}

/// 多链接口 trait
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    fn chain_type(&self) -> ChainType;
    fn chain_name(&self) -> &str;
    async fn record_swap(&self, proof: &SwapProof) -> Result<ChainRecord, BlockchainError>;
    async fn get_history(&self, item_id: &str) -> Result<Vec<ChainRecord>, BlockchainError>;
    async fn get_swap_count(&self, item_id: &str) -> Result<u64, BlockchainError>;
    async fn verify_transaction(&self, tx_hash: &str) -> Result<bool, BlockchainError>;
}

/// 多链管理器
pub struct ChainManager {
    adapters: Vec<Box<dyn ChainAdapter>>,
}

impl ChainManager {
    pub fn new() -> Self {
        Self { adapters: Vec::new() }
    }

    pub fn register(&mut self, adapter: Box<dyn ChainAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn get_adapter(&self, chain: &ChainType) -> Option<&dyn ChainAdapter> {
        self.adapters
            .iter()
            .find(|a| a.chain_type() == *chain)
            .map(|a| a.as_ref())
    }

    pub fn supported_chains(&self) -> Vec<ChainType> {
        self.adapters.iter().map(|a| a.chain_type()).collect()
    }

    pub async fn record_swap(
        &self,
        chain: &ChainType,
        proof: &SwapProof,
    ) -> Result<ChainRecord, BlockchainError> {
        let adapter = self
            .get_adapter(chain)
            .ok_or_else(|| BlockchainError::UnsupportedChain(format!("{chain:?}")))?;
        adapter.record_swap(proof).await
    }

    /// 在所有已注册链上存证（多链备份）
    pub async fn record_swap_all(
        &self,
        proof: &SwapProof,
    ) -> Result<Vec<ChainRecord>, BlockchainError> {
        let mut records = Vec::new();
        for adapter in &self.adapters {
            match adapter.record_swap(proof).await {
                Ok(record) => records.push(record),
                Err(e) => {
                    tracing::warn!("链 {} 存证失败: {}", adapter.chain_name(), e);
                }
            }
        }
        Ok(records)
    }
}

impl Default for ChainManager {
    fn default() -> Self {
        Self::new()
    }
}
