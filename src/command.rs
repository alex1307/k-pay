use std::{collections::HashMap, sync::RwLock};

use crate::model::{Balance, Transaction, TransactionType};

lazy_static! {
    pub static ref BALANCE_SHEET: RwLock<HashMap<i32, Balance>> = RwLock::new(HashMap::new());
    pub static ref TRANSACTIONS: RwLock<HashMap<i32, Transaction>> = RwLock::new(HashMap::new());
}

pub fn update_balance(transaction: Transaction, balance: &mut Vec<Balance>) {
    let mut balance_sheet = BALANCE_SHEET.write().unwrap();
    if let Some(mut current_balance) = balance_sheet.get_mut(&transaction.client) {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                current_balance.available += transaction.amount;
                current_balance.total += transaction.amount;
            }
            TransactionType::Withdrawal => {
                current_balance.available -= transaction.amount;
                current_balance.total -= transaction.amount;
            }
            TransactionType::Dispute => {
                current_balance.held += transaction.amount;
                current_balance.total += transaction.amount;
            }
            TransactionType::Resolve => {
                current_balance.held -= transaction.amount;
                current_balance.total -= transaction.amount;
            }
            TransactionType::Chargeback => {
                current_balance.held -= transaction.amount;
                current_balance.total -= transaction.amount;
            }
        }
    } else {
        if let Some(new_balance) = Transaction::try_from(&transaction) {
            balance_sheet.insert(transaction.client, new_balance);
        } else {
            println!("Unsupported transaction type");
        }
    };
}

fn dispute(tx: i32, balance: &mut Balance) {
    if let Some(transaction) = TRANSACTIONS.read().unwrap().get(&tx) {
        balance.held += transaction.amount;
        balance.available -= transaction.amount;
    } else {
        println!("Transaction not found");
    }
}

fn resolve(tx: i32, balance: &mut Balance) {
    if let Some(transaction) = TRANSACTIONS.read().unwrap().get(&tx) {
        balance.held -= transaction.amount;
        balance.available += transaction.amount;
    } else {
        println!("Transaction not found");
    }
}

fn chargeback(tx: i32, balance: &mut Balance) {
    if let Some(transaction) = TRANSACTIONS.read().unwrap().get(&tx) {
        balance.held -= transaction.amount;
        balance.total -= transaction.amount;
        balance.locked = true;
    } else {
        println!("Transaction not found");
    }
}
