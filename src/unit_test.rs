use crate::{model::Balances, streams::stream_lines};
use futures::StreamExt;

#[cfg(test)]
#[tokio::test]
async fn dispute_resolved_test() {
    let mut events = stream_lines("test_files/dispute_resolved.csv".to_string())
        .await
        .unwrap();
    let mut balances = Balances::new();
    while let Some(event) = events.next().await {
        println!("Event: {:?}", event);
        balances.process(event);
    }
    assert_eq!(1, balances.accounts.len());
    assert_eq!(50.00, balances.accounts.get(&1).unwrap().total);
    assert_eq!(50.00, balances.accounts.get(&1).unwrap().available);
    assert_eq!(0.00, balances.accounts.get(&1).unwrap().held);
    assert_eq!(false, balances.accounts.get(&1).unwrap().locked);
    assert_eq!(1, balances.accounts.get(&1).unwrap().client_id);
}

#[tokio::test]
async fn deposit_and_withdraw_test() {
    let mut events = stream_lines("test_files/deposit_withdraw.csv".to_string())
        .await
        .unwrap();
    let mut balances = Balances::new();
    while let Some(event) = events.next().await {
        println!("Event: {:?}", event);
        balances.process(event);
    }
    assert_eq!(1, balances.accounts.len());
    assert_eq!(100.00, balances.accounts.get(&1).unwrap().total);
    assert_eq!(100.00, balances.accounts.get(&1).unwrap().available);
    assert_eq!(0.00, balances.accounts.get(&1).unwrap().held);
    assert_eq!(false, balances.accounts.get(&1).unwrap().locked);
    assert_eq!(1, balances.accounts.get(&1).unwrap().client_id);
}

#[tokio::test]
async fn dispute_and_chargeback_test() {
    let mut events = stream_lines("test_files/dispute_chargeback.csv".to_string())
        .await
        .unwrap();
    let mut balances = Balances::new();
    while let Some(event) = events.next().await {
        println!("Event: {:?}", event);
        balances.process(event);
    }
    // The withdrawal happens twice, so the total is 0.00
    assert_eq!(1, balances.accounts.len());
    assert_eq!(0.00, balances.accounts.get(&1).unwrap().total);
    assert_eq!(0.00, balances.accounts.get(&1).unwrap().available);
    assert_eq!(0.00, balances.accounts.get(&1).unwrap().held);
    assert_eq!(true, balances.accounts.get(&1).unwrap().locked);
    assert_eq!(1, balances.accounts.get(&1).unwrap().client_id);
}
