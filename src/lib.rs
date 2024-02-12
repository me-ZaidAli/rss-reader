use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};
use clap::Parser;
use rss::{Channel, Item};
use std::{time::Duration, vec};
use tokio::{
    io::{self, AsyncBufReadExt},
    task::JoinSet,
    time::Instant,
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
    time_to_fetch: Duration,
    items: Vec<FeedItem>,
}

#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(short = 'd')]
    pub date: Option<NaiveDate>,
}

async fn read_feed_urls() -> Result<Vec<String>> {
    let mut urls: Vec<String> = vec![];

    let input = io::stdin();
    let reader = io::BufReader::new(input);
    let mut lines = reader.lines();

    let _ = lines.next_line().await;

    while let Some(line) = lines.next_line().await? {
        if let Some(url) = line.split(',').nth(0) {
            urls.push(url.to_string());
        }
    }

    Ok(urls)
}

struct FetchRssFeedResponse {
    channel: Channel,
    time_to_fetch: Duration,
}

async fn fetch_rss_feed(feed_url: &str) -> Result<FetchRssFeedResponse> {
    let now = Instant::now();
    let request_data = reqwest::get(feed_url)
        .await
        .context("Couldn't fetch rss feed")?;

    let latency = now.elapsed();

    let feed_content_bytes = request_data
        .bytes()
        .await
        .context("Can't convert request data to bytes")?;

    Ok(FetchRssFeedResponse {
        channel: Channel::read_from(&feed_content_bytes[..]).context("Invalid feed content")?,
        time_to_fetch: latency,
    })
}

fn transform(
    FetchRssFeedResponse {
        channel,
        time_to_fetch,
    }: FetchRssFeedResponse,
) -> Result<FeedChannel> {
    let feed_transformer = |item: Item| -> Result<FeedItem> {
        let pub_date = DateTime::parse_from_rfc2822(&item.pub_date.unwrap())
            .context("Date format isn't rfc2822 ")?
            .date_naive();

        Ok(FeedItem {
            title: item.title.unwrap(),
            pub_date,
        })
    };

    let transformed_feed_items = channel
        .items
        .into_iter()
        .filter(|item| item.pub_date.is_some() && item.title.is_some())
        .map(|item| feed_transformer(item))
        .collect::<Result<Vec<FeedItem>>>()?;

    Ok(FeedChannel {
        channel_name: channel.title,
        items: transformed_feed_items,
        time_to_fetch,
    })
}

async fn filter_feed_items_with(date: &NaiveDate, feed_to_filter: FeedChannel) -> FeedChannel {
    let items: Vec<FeedItem> = feed_to_filter.items;

    let filtered_items = items
        .into_iter()
        .filter(|item| {
            return item.pub_date >= *date;
        })
        .collect::<Vec<FeedItem>>();

    return FeedChannel {
        items: filtered_items,
        ..feed_to_filter
    };
}

pub async fn run(args: &Arguments) -> Result<()> {
    let date = args.date;

    let feed_urls = read_feed_urls().await?;

    let mut set: JoinSet<FeedChannel> = JoinSet::new();

    for url in feed_urls {
        set.spawn(async move {
            let response = fetch_rss_feed(&url).await.unwrap();

            let transformed_channel = transform(response).unwrap();

            if date.is_some() {
                return filter_feed_items_with(&date.unwrap(), transformed_channel).await;
            }

            transformed_channel
        });
    }

    while let Some(task) = set.join_next().await {
        match task {
            Ok(feed) => println!("{:#?}", feed),
            Err(e) => eprintln!("{}", e),
        }
    }

    Ok(())
}
