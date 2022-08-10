use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use csv::ReaderBuilder;
use futures::{
    channel::mpsc::{self, Sender},
    Stream, StreamExt,
};
use log::{error, info};
use tokio::{
    task,
    time::{sleep, Duration},
};

use crate::model::Event;
pub const CVS_SEPARATOR: u8 = b',';

pub async fn stream_lines(file_name: String) -> Result<impl Stream<Item = Event>, String> {
    let (mut sender, receiver) = mpsc::channel(100);
    task::spawn(async move {
        let file = File::open(&file_name).unwrap();
        info!("reading from file: {}", file_name);
        let reader = BufReader::new(file);
        let lines = reader.lines();
        let mut buffer = vec![];
        let mut counter = 1;
        for line in lines {
            if counter == 1 {
                counter += 1;
                continue;
            }
            match line {
                Ok(l) => {
                    buffer.push(l.clone());
                    if buffer.len() == 100 {
                        flush(&mut buffer, &mut sender).await;
                    }
                }
                Err(_) => error!("Can't read line # {}", counter),
            }
        }
        if !buffer.is_empty() {
            flush(&mut buffer, &mut sender).await;
        }
    });

    Ok(receiver)
}

async fn flush(buffer: &mut Vec<String>, sender: &mut Sender<Event>) {
    let data = buffer.join("\n");
    let mut rdr = ReaderBuilder::new()
        .delimiter(CVS_SEPARATOR)
        .has_headers(false)
        .from_reader(data.as_bytes());
    for result in rdr.deserialize() {
        let event: Event = match result {
            Ok(e) => e,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };
        if let Err(_err) = sender.try_send(event) {
            error!("Failed sending event.");
        }
    }
    sleep(Duration::from_millis(100)).await;
    buffer.clear();
}
