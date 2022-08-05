use std::{convert::TryFrom, borrow::BorrowMut, collections::HashMap};

use lazy_static::__Deref;

#[derive(Debug, PartialEq, Clone)]
pub struct TransactionEvent {
    pub id: i32,
    pub transaction_type: EventType,
    pub client: i32,
    pub amount: Option<f64>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum EventType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Dispute {
    pub transaction_id: i32,
    pub amount: f64,
    pub status: DisputeStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DisputeStatus {
    Open,
    Resolved,
    Chargeback,
}

pub struct ClientBalance {
    pub id: i32,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
    pub trnasactions: Vec<Transaction>,
    pub disputes: Vec<Dispute>,
}


pub struct Transaction {
    pub id: i32,
    pub transaction_type: TransactionType,
    pub client: i32,
    pub amount: f64,
}

pub enum TransactionType {
    Deposit,
    Withdrawal,
}


pub struct Balance {
    pub client: i32,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}


impl TryFrom<EventType> for DisputeStatus {
    type Error = &'static str;

    fn try_from(s: EventType) -> Result<Self, Self::Error> {
        match s {
            EventType::Dispute => Ok(DisputeStatus::Open),
            EventType::Resolve => Ok(DisputeStatus::Resolved),
            EventType::Chargeback => Ok(DisputeStatus::Chargeback),
            _ => Err("Unsupported transaction type"),
        }
    }
}

impl TryFrom<TransactionEvent> for Transaction {
    type Error = &'static str;

    fn try_from(s: TransactionEvent) -> Result<Self, Self::Error> {
        match s.transaction_type {
            EventType::Deposit => Ok(Transaction{
                id: s.id,
                transaction_type: TransactionType::Deposit,
                client: s.client,
                amount: s.amount.unwrap(),
            }),
            
            EventType::Withdrawal => Ok(Transaction{
                id: s.id,
                transaction_type: TransactionType::Withdrawal,
                client: s.client,
                amount: s.amount.unwrap(),
            }),
            _ => Err("Invalid transaction"),
        }
    }
    
}

impl ClientBalance {

    pub fn new(client_id: i32, tx_id: i32, amount: f64) -> Self {
        ClientBalance {
            id: client_id,
            available: amount,
            held: 0.0,
            total: amount,
            locked: false,
            trnasactions: vec![Transaction {
                id: tx_id,
                transaction_type: TransactionType::Deposit,
                client: client_id,
                amount: amount,
            }],
            disputes: vec![],
        }
    }

    

    pub fn withdrawal(&mut self, tx_id: i32, amount: f64) -> Result<(), &str> {
        if self.available < amount {
            return Err("Insufficient funds");
        }
        self.available -= amount;
        self.total -= amount;
        self.trnasactions.push(Transaction {
            id: tx_id,
            transaction_type: TransactionType::Withdrawal,
            client: self.id,
            amount,
        });
        Ok(())
    }

    pub fn deposit(&mut self, tx_id: i32, amount: f64) -> Result<(), &str> {
        
            self.available += amount;
            self.total += amount;
            self.trnasactions.push(Transaction {
                    id: tx_id,
                    transaction_type: TransactionType::Deposit,
                    client: self.id,
                    amount,
            });
            Ok(())
    }

    pub fn open_dispute(&mut self, tx_id: i32) -> Result<(), &str> {
        if let Some(_exist) =  self.disputes.iter().find(|t| t.transaction_id == tx_id) {
            Err("The dispute has been already opened")
        } else {
            if let Some(tx) = self.trnasactions.iter().find(|t| t.id == tx_id){
                let dispute = Dispute {
                    transaction_id: tx_id,
                    amount: tx.amount,
                    status: DisputeStatus::Open,
                };
                self.disputes.push(dispute);
                self.available -= tx.amount;
                self.held += tx.amount;
                Ok(())
            } else {
                Err("The transaction does not exist")
            }
        }
    }

    pub fn resolve(&mut self, tx_id: i32) -> Result<(), &str> {
        let disputes = self.disputes.iter().cloned().find(|t| t.transaction_id == tx_id);
        if let Some(opened) =  disputes {
            if opened.status == DisputeStatus::Open {
                let resolved = Dispute {
                    transaction_id: tx_id,
                    amount: opened.amount,
                    status: DisputeStatus::Resolved,
                };
                self.disputes.push(resolved);
                self.available += opened.amount;
                self.held -= opened.amount;
                Ok(())
            } else {
                Err("The dispute has been already resolved or chargebacked")
            }
        } else {
            Err("The dispute does not exist")
        }
    }

    pub fn chargeback(&mut self, tx_id: i32) -> Result<(), &str> {
        let disputes = self.disputes.iter().cloned().find(|t| t.transaction_id == tx_id);
        if let Some(opened) =  disputes {
            if opened.status == DisputeStatus::Open {
                let resolved = Dispute {
                    transaction_id: tx_id,
                    amount: opened.amount,
                    status: DisputeStatus::Chargeback,
                };
                self.disputes.push(resolved);
                self.total -= opened.amount;
                self.held = 0.0;
                self.locked = true;
                Ok(())
            } else {
                Err("The dispute has been already resolved or chargebacked")
            }
        } else {
            Err("The dispute does not exist")
        }
    }
       
}

#[test]
fn open_deposit() {
    let event = TransactionEvent{
        id: 1,
        transaction_type: EventType::Deposit,
        client: 1,
        amount: Some(100.0),
    };

    let client = ClientBalance::new(event.client, event.id, event.amount.unwrap());
    assert_eq!(100.0, client.available);
    assert_eq!(100.0, client.total);
    assert_eq!(0.0, client.held);
    assert_eq!(false, client.locked);
    assert_eq!(1, client.trnasactions.len());
    assert!(client.disputes.is_empty());
}

#[test]
fn make_deposit_and_withdraw() {
    let events = vec![TransactionEvent{
        id: 1,
        transaction_type: EventType::Deposit,
        client: 1,
        amount: Some(100.0),
    }, TransactionEvent{
        id: 2,
        transaction_type: EventType::Withdrawal,
        client: 1,
        amount: Some(50.0),
    }, TransactionEvent{
        id: 3,
        transaction_type: EventType::Deposit,
        client: 1,
        amount: Some(50.0),
    }, TransactionEvent{
        id: 4,
        transaction_type: EventType::Withdrawal,
        client: 1,
        amount: Some(20.0),
    }, TransactionEvent{
        id: 5,
        transaction_type: EventType::Deposit,
        client: 2,
        amount: Some(50.0),
    }, TransactionEvent{
        id: 6,
        transaction_type: EventType::Withdrawal,
        client: 2,
        amount: Some(25.0),
    }, TransactionEvent{
        id: 6,
        transaction_type: EventType::Withdrawal,
        client: 2,
        amount: Some(25.0),
    }];

    let mut clients = vec![];
    let mut transactions = vec![];

    for event in events{
        if transactions.contains(&event.id) {
            println!("Transaction {} already exists", event.id);
            continue;
        }
        let index_element = clients
            .iter()
            .position(|c:&ClientBalance| c.id == event.client);
        if let Some(idx) = index_element {
            if let Some(balance) = clients.get_mut(idx) {
                match event.transaction_type {
                    EventType::Deposit => {
                        balance.deposit(event.id, event.amount.unwrap()).unwrap();
                    }
                    EventType::Withdrawal => {
                        balance.withdrawal(event.id, event.amount.unwrap()).unwrap();
                    }
                    EventType::Dispute => todo!(),
                    EventType::Resolve => todo!(),
                    EventType::Chargeback => todo!(),
                }
            }    
        } else {
            let client = ClientBalance::new(event.client, event.id, event.amount.unwrap());
            clients.push(client);
        }
        transactions.push(event.id);
    }

    assert_eq!(2, clients.len());
    assert_eq!(80.0, clients[0].available);
    assert_eq!(4, clients[0].trnasactions.len());
    assert_eq!(80.0, clients[0].total);
    assert_eq!(0.0, clients[0].held);

    assert_eq!(25.0, clients[1].available);
    assert_eq!(2, clients[1].trnasactions.len());
    assert_eq!(25.0, clients[1].total);
    assert_eq!(0.0, clients[1].held);

}

