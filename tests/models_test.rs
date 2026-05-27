use chrono::Utc;
use mutao::models::*;
use uuid::Uuid;

fn make_swap_leg(from: u8, to: u8, offer: u8, want: u8) -> SwapLeg {
    SwapLeg {
        from_user_id: Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, from]),
        to_user_id: Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, to]),
        offer_item_id: Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, offer]),
        want_item_id: Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, want]),
    }
}

#[test]
fn test_swap_cycle_valid_two_person() {
    let cycle = SwapCycle {
        id: Uuid::new_v4(),
        swaps: vec![
            make_swap_leg(1, 2, 101, 102),
            make_swap_leg(2, 1, 102, 101),
        ],
        created_at: Utc::now(),
    };
    assert!(cycle.is_valid());
}

#[test]
fn test_swap_cycle_valid_three_person() {
    let cycle = SwapCycle {
        id: Uuid::new_v4(),
        swaps: vec![
            make_swap_leg(1, 2, 101, 102),
            make_swap_leg(2, 3, 102, 103),
            make_swap_leg(3, 1, 103, 101),
        ],
        created_at: Utc::now(),
    };
    assert!(cycle.is_valid());
}

#[test]
fn test_swap_cycle_invalid_single_leg() {
    let cycle = SwapCycle {
        id: Uuid::new_v4(),
        swaps: vec![make_swap_leg(1, 2, 101, 102)],
        created_at: Utc::now(),
    };
    assert!(!cycle.is_valid());
}

#[test]
fn test_swap_cycle_invalid_empty() {
    let cycle = SwapCycle {
        id: Uuid::new_v4(),
        swaps: vec![],
        created_at: Utc::now(),
    };
    assert!(!cycle.is_valid());
}

#[test]
fn test_swap_cycle_invalid_broken_chain() {
    let cycle = SwapCycle {
        id: Uuid::new_v4(),
        swaps: vec![
            make_swap_leg(1, 2, 101, 102),
            make_swap_leg(3, 1, 103, 101),
        ],
        created_at: Utc::now(),
    };
    assert!(!cycle.is_valid());
}

#[test]
fn test_item_status_serialization() {
    let statuses = vec![
        ItemStatus::Idle,
        ItemStatus::Matching,
        ItemStatus::Completed,
        ItemStatus::Archived,
    ];
    for s in statuses {
        let json = serde_json::to_string(&s).unwrap();
        let back: ItemStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}

#[test]
fn test_item_serialization_roundtrip() {
    let item = Item {
        id: Uuid::new_v4(),
        owner_id: Uuid::new_v4(),
        title: "机械键盘".into(),
        description: "用了两年".into(),
        image_url: "".into(),
        tags: vec!["键盘".into(), "外设".into()],
        value_tier: 3,
        status: ItemStatus::Idle,
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: Item = serde_json::from_str(&json).unwrap();
    assert_eq!(item.id, back.id);
    assert_eq!(item.title, back.title);
    assert_eq!(item.tags, back.tags);
    assert_eq!(item.value_tier, back.value_tier);
}

#[test]
fn test_demand_serialization_roundtrip() {
    let demand = Demand {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        offer_item_id: Uuid::new_v4(),
        offer_tags: vec!["书籍".into()],
        target_tags: vec!["键盘".into()],
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&demand).unwrap();
    let back: Demand = serde_json::from_str(&json).unwrap();
    assert_eq!(demand.id, back.id);
    assert_eq!(demand.offer_tags, back.offer_tags);
}
