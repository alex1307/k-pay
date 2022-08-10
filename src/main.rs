mod app;
mod model;
mod streams;

mod unit_test;

use std::{env, time::Duration};

use crate::app::process;
use log::{error, info};
use tokio::time::sleep;

pub static mut STATUS: [bool; 8] = [false; 8];

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut args: Vec<String> = env::args().skip(1).collect();
    info!("Args: {:?}", args);
    if args.is_empty() {
        error!("File name is required");
        return;
    }
    let file_name = args.pop().unwrap();
    println!(
        "{: >12}, {: >12}, {: >12}, {: >12}, {: >12}",
        "client", "available", "held", "total", "locked"
    );
    let _ = process(file_name, 8).await;
    unsafe {
        let mut status = true;
        for s in STATUS {
            status = s && status;
            if !status {
                sleep(Duration::from_millis(100)).await;
                continue;
            }
        }
        info!("Status: {:?}", STATUS);
    }
}
