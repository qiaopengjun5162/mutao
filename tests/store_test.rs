use chrono::Utc;
use mutao::models::*;
use mutao::store::Store;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_store() -> Store {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/mutao".into());
    let pool = PgPool::connect(&database_url).await.unwrap();
    Store::new(pool)
}

fn make_item(owner_id: Uuid, title: &str, tags: Vec<&str>, value_tier: u8) -> Item {
    Item {
        id: Uuid::new_v4(),
        owner_id,
        title: title.into(),
        description: "测试描述".into(),
        image_url: "".into(),
        tags: tags.into_iter().map(String::from).collect(),
        value_tier,
        status: ItemStatus::Idle,
        created_at: Utc::now(),
    }
}

fn make_demand(user_id: Uuid, offer_item_id: Uuid, offer: Vec<&str>, want: Vec<&str>) -> Demand {
    Demand {
        id: Uuid::new_v4(),
        user_id,
        offer_item_id,
        offer_tags: offer.into_iter().map(String::from).collect(),
        target_tags: want.into_iter().map(String::from).collect(),
        created_at: Utc::now(),
    }
}

fn make_user(username: &str) -> User {
    User {
        id: Uuid::new_v4(),
        username: username.into(),
        password_hash: "$2b$12$fakehash".into(),
        created_at: Utc::now(),
    }
}

// ---- Item CRUD ----

#[tokio::test]
async fn test_create_and_get_item() {
    let store = setup_store().await;
    let owner = Uuid::new_v4();
    let item = make_item(owner, "机械键盘", vec!["键盘", "外设"], 3);

    store.create_item(&item).await.unwrap();

    let fetched = store.get_item(item.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, item.id);
    assert_eq!(fetched.title, "机械键盘");
    assert_eq!(fetched.tags, vec!["键盘", "外设"]);
    assert_eq!(fetched.value_tier, 3);
    assert_eq!(fetched.status, ItemStatus::Idle);
}

#[tokio::test]
async fn test_get_item_not_found() {
    let store = setup_store().await;
    let result = store.get_item(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_list_items() {
    let store = setup_store().await;
    let items = store.list_items().await.unwrap();
    // 列表查询成功即可，不依赖其他测试的数据
    let _ = items.len();
}

#[tokio::test]
async fn test_item_exists() {
    let store = setup_store().await;
    let owner = Uuid::new_v4();
    let item = make_item(owner, "书籍", vec!["书"], 1);

    assert!(!store.item_exists(item.id).await.unwrap());

    store.create_item(&item).await.unwrap();

    assert!(store.item_exists(item.id).await.unwrap());
}

#[tokio::test]
async fn test_update_item_status() {
    let store = setup_store().await;
    let owner = Uuid::new_v4();
    let item = make_item(owner, "音箱", vec!["音频"], 4);

    store.create_item(&item).await.unwrap();
    assert!(store.update_item_status(item.id, &ItemStatus::Matching).await.unwrap());

    let updated = store.get_item(item.id).await.unwrap().unwrap();
    assert_eq!(updated.status, ItemStatus::Matching);
}

#[tokio::test]
async fn test_update_item_status_not_found() {
    let store = setup_store().await;
    let changed = store.update_item_status(Uuid::new_v4(), &ItemStatus::Matching).await.unwrap();
    assert!(!changed);
}

// ---- Demand CRUD ----

#[tokio::test]
async fn test_create_and_list_demands() {
    let store = setup_store().await;
    let user_id = Uuid::new_v4();
    let offer_item_id = Uuid::new_v4();
    let demand = make_demand(user_id, offer_item_id, vec!["书籍"], vec!["键盘"]);

    store.create_demand(&demand).await.unwrap();

    let demands = store.list_demands().await.unwrap();
    assert!(demands.iter().any(|d| d.id == demand.id));
}

// ---- Cycle CRUD ----

#[tokio::test]
async fn test_save_and_list_cycles() {
    let store = setup_store().await;
    let cycle = SwapCycle {
        id: Uuid::new_v4(),
        swaps: vec![
            SwapLeg {
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                offer_item_id: Uuid::new_v4(),
                want_item_id: Uuid::new_v4(),
            },
            SwapLeg {
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                offer_item_id: Uuid::new_v4(),
                want_item_id: Uuid::new_v4(),
            },
        ],
        created_at: Utc::now(),
    };

    store.save_cycle(&cycle).await.unwrap();

    let cycles = store.list_cycles().await.unwrap();
    assert!(cycles.iter().any(|c| c.id == cycle.id && c.swaps.len() == 2));
}

// ---- User CRUD ----

#[tokio::test]
async fn test_create_and_get_user() {
    let store = setup_store().await;
    let username = format!("test_{}", Uuid::new_v4().as_simple());
    let user = make_user(&username);

    store.create_user(&user).await.unwrap();

    let fetched = store.get_user_by_username(&username).await.unwrap().unwrap();
    assert_eq!(fetched.id, user.id);
    assert_eq!(fetched.username, username);
    assert_eq!(fetched.password_hash, "$2b$12$fakehash");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let store = setup_store().await;
    let result = store.get_user_by_username("nobody_at_all").await.unwrap();
    assert!(result.is_none());
}
