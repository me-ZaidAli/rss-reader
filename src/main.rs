use clap::Parser;
use log::error;
use rssreader::{run, Arguments, FeedChannel};
use tokio::{sync::mpsc::unbounded_channel, task};

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();
    let (tx, mut rx) = unbounded_channel::<FeedChannel>();

    task::spawn(async move {
        if let Err(e) = run(&arguments, &tx).await {
            error!("Application error: {}", e);
        }
    });

    while let Some(feed) = rx.recv().await {
        println!("{:#?}", feed);
    }

    rx.close();
}
