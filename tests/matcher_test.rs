use mutao::matcher::Matcher;
use mutao::models::Demand;
use uuid::Uuid;

fn demand(user: u8, item: u8, offer: Vec<&str>, want: Vec<&str>) -> Demand {
    Demand {
        id: Uuid::new_v4(),
        user_id: Uuid::from_bytes([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,user]),
        offer_item_id: Uuid::from_bytes([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,item]),
        offer_tags: offer.into_iter().map(String::from).collect(),
        target_tags: want.into_iter().map(String::from).collect(),
        created_at: chrono::Utc::now(),
    }
}

#[test]
fn test_two_person_cycle() {
    let d = vec![
        demand(1, 101, vec!["书籍"], vec!["键盘"]),
        demand(2, 102, vec!["键盘"], vec!["书籍"]),
    ];
    let cycles = Matcher::find_cycles(&d, 4);
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].swaps.len(), 2);
}

#[test]
fn test_three_person_cycle() {
    let d = vec![
        demand(1, 101, vec!["书籍"], vec!["键盘"]),
        demand(2, 102, vec!["键盘"], vec!["耳机"]),
        demand(3, 103, vec!["耳机"], vec!["书籍"]),
    ];
    let cycles = Matcher::find_cycles(&d, 4);
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].swaps.len(), 3);
}

#[test]
fn test_no_cycle_single() {
    let d = vec![demand(1, 1, vec!["书籍"], vec!["键盘"])];
    assert!(Matcher::find_cycles(&d, 4).is_empty());
}

#[test]
fn test_no_cycle_mismatch() {
    let d = vec![
        demand(1, 101, vec!["书籍"], vec!["键盘"]),
        demand(2, 102, vec!["书籍"], vec!["耳机"]),
    ];
    assert!(Matcher::find_cycles(&d, 4).is_empty());
}

#[test]
fn test_two_separate_cycles() {
    let d = vec![
        demand(1, 101, vec!["书籍"], vec!["键盘"]),
        demand(2, 102, vec!["键盘"], vec!["书籍"]),
        demand(3, 103, vec!["耳机"], vec!["音箱"]),
        demand(4, 104, vec!["音箱"], vec!["耳机"]),
    ];
    assert_eq!(Matcher::find_cycles(&d, 4).len(), 2);
}
