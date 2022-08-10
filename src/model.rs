use std::collections::HashMap;

use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub event_type: EventType,
    pub transaction_id: u64,
    pub client_id: u64,
    pub amount: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum EventType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub client_id: u64,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
    pub transactions: HashMap<u64, f32>,
    pub open_disputes: Vec<u64>,
    pub resolved: Vec<u64>,
    pub chargedbacks: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: u64,
    pub amount: f32,
}

pub enum Command {
    Deposit(Transaction),
    Withdrawal(Transaction),
    Dispute(Transaction),
    Resolve(Transaction),
    Chargeback(Transaction),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Balances {
    pub accounts: HashMap<u64, Account>,
}

impl TryFrom<&str> for EventType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<EventType, Self::Error> {
        match value.to_lowercase().as_str() {
            "deposit" => Ok(EventType::Deposit),
            "withdrawal" => Ok(EventType::Withdrawal),
            "dispute" => Ok(EventType::Dispute),
            "resolve" => Ok(EventType::Resolve),
            "chargeback" => Ok(EventType::Chargeback),
            _ => Err("Invalid event type"),
        }
    }
}

impl Account {
    pub fn new(client_id: u64, transaction_id: u64, amount: f32) -> Self {
        let mut transactions = HashMap::new();
        transactions.insert(transaction_id, amount);
        Account {
            client_id,
            available: amount,
            held: 0.0,
            total: amount,
            locked: false,
            transactions,
            open_disputes: vec![],
            resolved: vec![],
            chargedbacks: vec![],
        }
    }

    pub fn execute(&mut self, command: Command) -> Result<(), &str> {
        match command {
            Command::Deposit(tx) => {
                if self.transactions.contains_key(&tx.id) {
                    return Err("Deposit failed. Transaction already exists.");
                }

                self.total += tx.amount;
                self.available += tx.amount;
                self.transactions.insert(tx.id, tx.amount);
            }
            Command::Withdrawal(tx) => {
                if self.transactions.contains_key(&tx.id) {
                    return Err("Withdraw failed. Transaction already exists.");
                }

                if tx.amount <= self.available {
                    self.total -= tx.amount;
                    self.available -= tx.amount;
                    self.transactions.insert(tx.id, tx.amount);
                } else {
                    return Err("Unsufficient funds.");
                }
            }
            Command::Dispute(tx) => {
                if self.available < tx.amount {
                    return Err("Dispute can't be opened. Unsufficient funds");
                }

                if !self.transactions.contains_key(&tx.id) {
                    return Err("Transaction not found");
                }

                self.available -= tx.amount;
                self.held += tx.amount;
                self.open_disputes.push(tx.id);
            }
            Command::Resolve(tx) => {
                if let Some(idx) = self.open_disputes.iter().position(|tx_id| tx_id == &tx.id) {
                    self.available += tx.amount;
                    self.held -= tx.amount;
                    self.open_disputes.remove(idx);
                    self.resolved.push(tx.id);
                } else {
                    return Err("Dispute not found");
                }
            }
            Command::Chargeback(tx) => {
                if let Some(idx) = self.open_disputes.iter().position(|tx_id| tx_id == &tx.id) {
                    self.total -= tx.amount;
                    self.held -= tx.amount;
                    self.open_disputes.remove(idx);
                    self.locked = true;
                    self.chargedbacks.push(tx.id);
                } else {
                    return Err("Dispute not found");
                }
            }
        }
        Ok(())
    }

    pub fn transaction(&self, transaction_id: u64) -> Option<Transaction> {
        self.transactions
            .get(&transaction_id)
            .map(|amt| Transaction {
                id: transaction_id,
                amount: *amt,
            })
    }
}

impl Balances {
    pub fn new() -> Self {
        Balances {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, event: Event) {
        let client_id = &event.client_id;
        if let Some(account) = self.accounts.get_mut(client_id) {
            let res = match event.event_type {
                EventType::Deposit => account.execute(Command::Deposit(Transaction {
                    id: event.transaction_id,
                    amount: event.amount.unwrap(),
                })),
                EventType::Withdrawal => account.execute(Command::Withdrawal(Transaction {
                    id: event.transaction_id,
                    amount: event.amount.unwrap(),
                })),
                EventType::Dispute => {
                    if let Some(tx) = account.transaction(event.transaction_id) {
                        account.execute(Command::Dispute(tx))
                    } else {
                        Err("Dispute failed. Transaction not found")
                    }
                }
                EventType::Resolve => {
                    if let Some(transaction) = account.transaction(event.transaction_id) {
                        account.execute(Command::Resolve(transaction))
                    } else {
                        Err("Resolve failed. Transaction not found")
                    }
                }
                EventType::Chargeback => {
                    if let Some(transaction) = account.transaction(event.transaction_id) {
                        account.execute(Command::Chargeback(transaction))
                    } else {
                        Err("Chargeback failed. Transaction not found")
                    }
                }
            };
            if res.is_err() {
                info!("Error: {}", res.err().unwrap());
            }
        } else if event.event_type == EventType::Deposit {
            info!("New account: {}", client_id);
            let account = Account::new(*client_id, event.transaction_id, event.amount.unwrap());
            self.accounts.insert(event.client_id, account);
        }
    }
}

#[test]
fn new_account_test() {
    let account = Account::new(1, 1, 100.00);
    assert_eq!(1, account.client_id);
    assert_eq!(100.0, account.available);
    assert_eq!(100.0, account.total);
    assert_eq!(0.0, account.held);
}

#[test]
fn make_account_and_deposit_test() {
    let mut account = Account::new(1, 1, 100.00);
    let tx = Transaction {
        id: 2,
        amount: 200.00,
    };

    let status = account.execute(Command::Deposit(tx));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(300.0, account.available);
    assert_eq!(300.0, account.total);
    assert_eq!(0.0, account.held);
    assert_eq!(2, account.transactions.len());
}

#[test]
fn make_account_and_withdraw_test() {
    let mut account = Account::new(1, 1, 100.00);
    let tx = Transaction {
        id: 2,
        amount: 50.00,
    };

    let status = account.execute(Command::Withdrawal(tx));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(50.0, account.available);
    assert_eq!(50.0, account.total);
    assert_eq!(0.0, account.held);
    assert_eq!(2, account.transactions.len());
}

#[test]
fn tranasaction_not_found_test() {
    let mut account = Account::new(1, 1, 100.00);
    let tx = Transaction {
        id: 2,
        amount: 50.00,
    };

    let status = account.execute(Command::Dispute(tx));
    assert!(status.is_err());
}

#[test]
fn open_dispute_test() {
    let mut account = Account::new(1, 1, 100.00);
    let tx = Transaction {
        id: 1,
        amount: 50.00,
    };
    let status = account.execute(Command::Dispute(tx));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(50.0, account.available);
    assert_eq!(100.0, account.total);
    assert_eq!(50.0, account.held);
    assert_eq!(1, account.transactions.len());
    assert_eq!(1, account.open_disputes.len());
}

#[test]
fn resolve_dispute_test() {
    let mut account = Account::new(1, 1, 100.00);
    let tx = Transaction {
        id: 1,
        amount: 50.00,
    };
    let status = account.execute(Command::Dispute(tx.clone()));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(50.0, account.available);
    assert_eq!(100.0, account.total);
    assert_eq!(50.0, account.held);
    assert_eq!(1, account.transactions.len());
    assert_eq!(1, account.open_disputes.len());

    let status = account.execute(Command::Resolve(tx));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(100.0, account.available);
    assert_eq!(100.0, account.total);
    assert_eq!(0.0, account.held);
    assert_eq!(1, account.transactions.len());
    assert_eq!(0, account.open_disputes.len());
    assert_eq!(1, account.resolved.len());
}

#[test]
fn chargeback_dispute_test() {
    let mut account = Account::new(1, 1, 100.00);
    let tx = Transaction {
        id: 1,
        amount: 50.00,
    };
    let status = account.execute(Command::Dispute(tx.clone()));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(50.0, account.available);
    assert_eq!(100.0, account.total);
    assert_eq!(50.0, account.held);
    assert_eq!(1, account.transactions.len());
    assert_eq!(1, account.open_disputes.len());

    let status = account.execute(Command::Chargeback(tx));
    assert!(status.is_ok());
    assert_eq!(1, account.client_id);
    assert_eq!(50.0, account.available);
    assert_eq!(50.0, account.total);
    assert_eq!(0.0, account.held);
    assert!(account.locked);
    assert_eq!(1, account.transactions.len());
    assert_eq!(0, account.open_disputes.len());
    assert_eq!(1, account.chargedbacks.len());
}

#[test]
fn balance_test() {
    let mut balances = Balances::new();
    let mut events = vec![];
    for i in 1..100 {
        let event = if i % 3 == 0 {
            Event {
                event_type: EventType::Withdrawal,
                transaction_id: i,
                client_id: i % 10 + 1,
                amount: Some(i as f32),
            }
        } else {
            Event {
                event_type: EventType::Deposit,
                transaction_id: i,
                client_id: i % 10 + 1,
                amount: Some(i as f32),
            }
        };

        events.push(event);
    }

    for e in events {
        balances.process(e);
    }

    assert_eq!(10, balances.accounts.len());
    for acc in balances.accounts.values() {
        println!("{:?}", acc);
    }
}
