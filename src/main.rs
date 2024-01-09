use clap::Parser;
use tokio::task::{self};

use rssreader::{run, Arguments};

#[tokio::main]
async fn main() {
    let arguments = task::spawn_blocking(|| Arguments::parse()).await;

    run(&arguments.unwrap()).await;
}
