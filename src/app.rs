use futures::StreamExt;
use log::{error, info};
use tokio::{
    sync::mpsc::{channel, Sender},
    task,
};

use crate::{
    model::{Balances, Event},
    streams::stream_lines,
    STATUS,
};

pub async fn process(file_name: String, workers_count: usize) -> Result<(), String> {
    let mut events = stream_lines(file_name).await?;
    let mut senders: Vec<Sender<Event>> = vec![];
    for i in 0..workers_count {
        let sender = executor(i).await;
        info!("Executor {} has been started", i);
        senders.push(sender);
    }

    while let Some(event) = events.next().await {
        let client_id = event.client_id;
        let transaction_id = event.transaction_id;
        let index = client_id % (workers_count as u64);
        let sender = &senders[index as usize];

        if let Err(err) = sender.try_send(event) {
            error!(
                "Failed sending event for transaction: {}. Err: {}",
                transaction_id, err
            );
        }
    }
    Ok(())
}

async fn executor(id: usize) -> Sender<Event> {
    let (tx, mut rx) = channel(100);
    task::spawn(async move {
        let mut balances = Balances::new();
        while let Some(event) = rx.recv().await {
            balances.process(event);
        }
        info!("All accounts: {}", balances.accounts.len());
        for acc in balances.accounts.values() {
            info!(
                "-->Executor #{} client_id {}, available: {}, held: {}, total: {}",
                id, acc.client_id, acc.available, acc.held, acc.total
            );
            println!(
                "{: >12}, {:>12.4}, {:>12.4}, {:>12.4}, {: >12}",
                acc.client_id, acc.available, acc.held, acc.total, acc.locked
            );
        }
        unsafe {
            STATUS[id] = true;
        }
    });
    tx
}
