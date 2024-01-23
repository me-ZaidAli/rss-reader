use clap::Parser;
use rssreader::{run, Arguments};

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    if let Err(e) = run(&arguments).await {
        eprintln!("Application error: {}", e);
    }
}
