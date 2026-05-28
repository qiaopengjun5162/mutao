use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Demand, Item, ItemStatus, SwapCycle, User};

/// 数据访问层，封装所有 PostgreSQL 操作
pub struct Store {
    pool: PgPool,
}

impl Store {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_item(&self, item: &Item) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO items (id, owner_id, title, description, image_url, tags, value_tier, status, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(item.id)
        .bind(item.owner_id)
        .bind(&item.title)
        .bind(&item.description)
        .bind(&item.image_url)
        .bind(&item.tags)
        .bind(item.value_tier as i16)
        .bind(format!("{:?}", item.status))
        .bind(item.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_item(&self, id: Uuid) -> Result<Option<Item>, sqlx::Error> {
        let row = sqlx::query_as::<_, ItemRow>(
            r#"SELECT id, owner_id, title, description, image_url, tags, value_tier, status, created_at
               FROM items WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into_item()))
    }

    pub async fn list_items(&self) -> Result<Vec<Item>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ItemRow>(
            r#"SELECT id, owner_id, title, description, image_url, tags, value_tier, status, created_at
               FROM items ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_item()).collect())
    }

    pub async fn create_demand(&self, demand: &Demand) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO demands (id, user_id, offer_item_id, offer_tags, target_tags, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(demand.id)
        .bind(demand.user_id)
        .bind(demand.offer_item_id)
        .bind(&demand.offer_tags)
        .bind(&demand.target_tags)
        .bind(demand.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_demands(&self) -> Result<Vec<Demand>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DemandRow>(
            r#"SELECT id, user_id, offer_item_id, offer_tags, target_tags, created_at
               FROM demands ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_demand()).collect())
    }

    pub async fn save_cycle(&self, cycle: &SwapCycle) -> Result<(), sqlx::Error> {
        let data = serde_json::to_value(cycle).map_err(sqlx::Error::decode)?;
        sqlx::query("INSERT INTO swap_cycles (id, data, created_at) VALUES ($1, $2, $3)")
            .bind(cycle.id)
            .bind(&data)
            .bind(cycle.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_cycles(&self) -> Result<Vec<SwapCycle>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CycleRow>(
            r#"SELECT id, data, created_at FROM swap_cycles ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let cycles: Vec<SwapCycle> = rows
            .into_iter()
            .filter_map(|r| {
                let mut cycle: SwapCycle = match serde_json::from_value(r.data) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("反序列化 swap_cycle 失败, id={}, err={}", r.id, e);
                        return None;
                    }
                };
                cycle.id = r.id;
                Some(cycle)
            })
            .collect();

        Ok(cycles)
    }

    pub async fn item_exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM items WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    pub async fn update_item_status(
        &self,
        id: Uuid,
        status: &ItemStatus,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE items SET status = $1 WHERE id = $2")
            .bind(format!("{:?}", status))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ---- User ----

    pub async fn create_user(&self, user: &User) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO users (id, username, password_hash, created_at)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, username, password_hash, created_at FROM users WHERE username = $1"#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into_user()))
    }
}

// ---- Database Row types ----

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    owner_id: Uuid,
    title: String,
    description: String,
    image_url: String,
    tags: Vec<String>,
    value_tier: i16,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl ItemRow {
    fn into_item(self) -> Item {
        Item {
            id: self.id,
            owner_id: self.owner_id,
            title: self.title,
            description: self.description,
            image_url: self.image_url,
            tags: self.tags,
            value_tier: self.value_tier as u8,
            status: match self.status.as_str() {
                "Matching" => ItemStatus::Matching,
                "Completed" => ItemStatus::Completed,
                "Archived" => ItemStatus::Archived,
                _ => ItemStatus::Idle,
            },
            created_at: self.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DemandRow {
    id: Uuid,
    user_id: Uuid,
    offer_item_id: Uuid,
    offer_tags: Vec<String>,
    target_tags: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DemandRow {
    fn into_demand(self) -> Demand {
        Demand {
            id: self.id,
            user_id: self.user_id,
            offer_item_id: self.offer_item_id,
            offer_tags: self.offer_tags,
            target_tags: self.target_tags,
            created_at: self.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CycleRow {
    id: Uuid,
    data: serde_json::Value,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    password_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl UserRow {
    fn into_user(self) -> User {
        User {
            id: self.id,
            username: self.username,
            password_hash: self.password_hash,
            created_at: self.created_at,
        }
    }
}
