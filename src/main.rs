use clap::Parser;
use tokio::task::{self};

use rssreader::{run, Arguments};

#[tokio::main]
async fn main() {
    let arguments = task::spawn_blocking(|| Arguments::parse()).await;

    if let Err(e) = run(&arguments.unwrap()).await {
        eprintln!("Application error: {}", e);
    }
}
