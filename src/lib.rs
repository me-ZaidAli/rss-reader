use chrono::{DateTime, NaiveDate};
use clap::Parser;
use rss::Channel;
use std::{sync::Arc, vec};
use tokio::{
    io::{self, AsyncBufReadExt},
    task::JoinSet,
};

mod tests;

#[derive(Debug, PartialEq, Clone)]
struct FeedItem {
    title: String,
    pub_date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
struct FeedChannel {
    channel_name: String,
    items: Vec<FeedItem>,
}

#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(short = 'd')]
    pub date: Option<NaiveDate>,
}

async fn read_feed_urls() -> Vec<String> {
    let mut urls: Vec<String> = vec![];

    let input = io::stdin();
    let reader = io::BufReader::new(input);
    let mut lines = reader.lines();

    let _ = lines.next_line().await;

    while let Some(line) = lines.next_line().await.expect("failed to read input") {
        let url = line.split(',').nth(0).unwrap();

        urls.push(url.to_string())
    }

    urls
}

async fn fetch_feed(feed_url: &str) -> Channel {
    let content = reqwest::get(feed_url).await.unwrap().bytes().await.unwrap();

    Channel::read_from(&content[..]).unwrap()
}

async fn transform(channel: Channel) -> FeedChannel {
    FeedChannel {
        channel_name: channel.title,
        items: channel
            .items
            .into_iter()
            .filter(|item| item.pub_date.is_some() && item.title.is_some())
            .map(|item| {
                return FeedItem {
                    title: item.title.unwrap(),
                    pub_date: DateTime::parse_from_rfc2822(&item.pub_date.unwrap())
                        .unwrap()
                        .date_naive(),
                };
            })
            .collect::<Vec<FeedItem>>(),
    }
}

async fn filter_with(date: &NaiveDate, feed_to_filer: FeedChannel) -> FeedChannel {
    let items = feed_to_filer.items;

    let filtered_items = items
        .into_iter()
        .filter(|item| {
            return item.pub_date >= *date;
        })
        .collect::<Vec<FeedItem>>();

    return FeedChannel {
        items: filtered_items,
        ..feed_to_filer
    };
}

pub async fn run(args: &Arguments) {
    let date = args.date;
    let arc_date = Arc::new(date);

    let feed_urls = read_feed_urls().await;

    let mut set: JoinSet<FeedChannel> = JoinSet::new();

    for url in feed_urls {
        let cloned_date = arc_date.clone();

        set.spawn(async move {
            let channel = fetch_feed(&url).await;
            let tranformed_channel = transform(channel).await;

            if cloned_date.is_some() {
                return filter_with(&cloned_date.unwrap(), tranformed_channel).await;
            }

            tranformed_channel
        });
    }

    while let Some(task) = set.join_next().await {
        match task {
            Ok(feed) => {
                if feed.items.len() < 10 {
                    println!("{:#?}", feed)
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}
