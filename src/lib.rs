use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};
use clap::Parser;
use reqwest::Url;
use rss::{Channel, Item};
use std::{time::Duration, vec};
use tokio::{
    io::{self, AsyncBufReadExt},
    task::JoinSet,
    time::Instant,
};

mod tests;

#[derive(Debug, PartialEq)]
struct FeedItem {
    title: String,
    pub_date: NaiveDate,
}

#[derive(Debug, PartialEq)]
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

async fn read_feed_urls() -> Result<Vec<Url>> {
    let mut urls: Vec<Url> = vec![];

    let input = io::stdin();
    let reader = io::BufReader::new(input);
    let mut lines = reader.lines();

    let _ = lines.next_line().await;

    while let Some(line) = lines.next_line().await? {
        if let Some(url) = line.split(',').nth(0) {
            let parsed_url =
                Url::parse(url).with_context(|| format!("Couldn't parse the url {}.", url))?;
            urls.push(parsed_url);
        }
    }

    Ok(urls)
}

struct RssResponse {
    channel: Channel,
    time_to_fetch: Duration,
}

async fn fetch_rss_feed(feed_url: &Url) -> Result<RssResponse> {
    let now = Instant::now();
    let request_data = reqwest::get(feed_url.as_str())
        .await
        .with_context(|| format!("Couldn't fetch rss feed from {}", feed_url))?;

    let latency = now.elapsed();

    let feed_content_bytes = request_data
        .bytes()
        .await
        .context("Can't convert request data to bytes")?;

    Ok(RssResponse {
        channel: Channel::read_from(&feed_content_bytes[..])
            .with_context(|| format!("Invalid feed content from {}", feed_url))?,
        time_to_fetch: latency,
    })
}

fn transform_feed_channel(
    RssResponse {
        channel,
        time_to_fetch,
    }: RssResponse,
) -> Result<FeedChannel> {
    let feed_transformer = |item: Item| -> Result<FeedItem> {
        let pub_date = DateTime::parse_from_rfc2822(&item.pub_date.unwrap())
            .context("Couldn't parse feed's published date. Rfc2822 date format needed.")?
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

    let mut set: JoinSet<RssResponse> = JoinSet::new();

    for url in feed_urls {
        set.spawn(async move { fetch_rss_feed(&url).await.unwrap() });
    }

    while let Some(task_response) = set.join_next().await {
        match task_response {
            Ok(response) => {
                let transformed_channel = transform_feed_channel(response).unwrap();

                println!(
                    "{:#?}",
                    if date.is_some() {
                        filter_feed_items_with(&date.unwrap(), transformed_channel).await
                    } else {
                        transformed_channel
                    }
                )
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    Ok(())
}
