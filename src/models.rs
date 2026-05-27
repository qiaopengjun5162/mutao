use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 物品状态
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ItemStatus {
    Idle,
    Matching,
    Completed,
    Archived,
}

impl ItemStatus {
    /// 检查状态转换是否合法
    pub fn can_transition_to(&self, target: &ItemStatus) -> bool {
        matches!(
            (self, target),
            (ItemStatus::Idle, ItemStatus::Matching)
                | (ItemStatus::Matching, ItemStatus::Completed)
                | (ItemStatus::Matching, ItemStatus::Idle)
                | (ItemStatus::Completed, ItemStatus::Archived)
                | (_, ItemStatus::Archived)
        )
    }

    /// 获取可用的下一个状态
    pub fn available_transitions(&self) -> Vec<ItemStatus> {
        match self {
            ItemStatus::Idle => vec![ItemStatus::Matching],
            ItemStatus::Matching => vec![ItemStatus::Completed, ItemStatus::Idle],
            ItemStatus::Completed => vec![ItemStatus::Archived],
            ItemStatus::Archived => vec![],
        }
    }
}

/// 核心实体：闲置物品
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: String,
    pub image_url: String,
    pub tags: Vec<String>,
    pub value_tier: u8,
    pub status: ItemStatus,
    pub created_at: DateTime<Utc>,
}

/// 交换意向：我想用我的 A，换别人手里什么样的 B
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Demand {
    pub id: Uuid,
    pub user_id: Uuid,
    pub offer_item_id: Uuid,
    pub offer_tags: Vec<String>,     // 我的物品的标签（匹配给别人用）
    pub target_tags: Vec<String>,    // 我想要换到的物品标签
    pub created_at: DateTime<Utc>,
}

/// 交换环：A 换 B, B 换 C, C 换 A
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapCycle {
    pub id: Uuid,
    pub swaps: Vec<SwapLeg>,
    pub created_at: DateTime<Utc>,
}

/// 一对一的交换腿
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapLeg {
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub offer_item_id: Uuid,
    pub want_item_id: Uuid,
}

impl SwapCycle {
    pub fn is_valid(&self) -> bool {
        if self.swaps.len() < 2 {
            return false;
        }
        self.swaps.iter().enumerate().all(|(i, current)| {
            let next = &self.swaps[(i + 1) % self.swaps.len()];
            current.to_user_id == next.from_user_id
        })
    }
}

/// 用户
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}
