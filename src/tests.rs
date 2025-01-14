#[cfg(test)]

mod tests {
    use std::time::Duration;
    use crate::{
        fetch_rss_feed, filter_feed_items_with, transform_feed_channel, FeedChannel, FeedItem,
        RssResponse,
    };
    use chrono::NaiveDate;
    use reqwest::Url;
    use rss::{ChannelBuilder, Item};

    #[tokio::test]
    async fn filter_items_with_date() {
        let mut expected = vec![FeedItem {
            title: String::from("1"),
            pub_date: NaiveDate::from_ymd_opt(2021, 01, 01).unwrap(),
        }];

        let feeds = mock_fetch_feeds().await;

        let mut filtered_feed =
            filter_feed_items_with(&NaiveDate::from_ymd_opt(2020, 12, 30).unwrap(), feeds).await;

        assert!(filtered_feed.items.iter().eq(expected.iter()));

        expected = vec![];

        let feeds = mock_fetch_feeds().await;

        filtered_feed = filter_feed_items_with(
            &NaiveDate::from_ymd_opt(2023, 12, 30).unwrap(),
            feeds,
        )
        .await;

        assert!(filtered_feed.items.iter().eq(expected.iter()));

        expected = vec![
            FeedItem {
                title: String::from("1"),
                pub_date: NaiveDate::from_ymd_opt(2021, 01, 01).unwrap(),
            },
            FeedItem {
                title: String::from("2"),
                pub_date: NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
            },
            FeedItem {
                title: String::from("3"),
                pub_date: NaiveDate::from_ymd_opt(2019, 01, 01).unwrap(),
            },
            FeedItem {
                title: String::from("4"),
                pub_date: NaiveDate::from_ymd_opt(2018, 01, 01).unwrap(),
            },
        ];

        let feeds = mock_fetch_feeds().await;

        filtered_feed = filter_feed_items_with(
            &NaiveDate::from_ymd_opt(2010, 12, 30).unwrap(),
            feeds,
        )
        .await;

        assert!(filtered_feed.items.iter().eq(expected.iter()));
    }

    #[tokio::test]
    async fn successful_channel_transformation() {
        let expected = FeedChannel {
            channel_name: "dummy channel".to_string(),
            time_to_fetch: Duration::new(2, 0),
            items: vec![
                FeedItem {
                    title: String::from("1"),
                    pub_date: NaiveDate::from_ymd_opt(2021, 10, 06).unwrap(),
                },
                FeedItem {
                    title: String::from("2"),
                    pub_date: NaiveDate::from_ymd_opt(2020, 10, 06).unwrap(),
                },
            ],
        };

        let mut item1 = Item::default();
        item1.set_title("1".to_string());
        item1.set_pub_date("Wed, 06 Oct 2021 17:00:53 GMT".to_string());

        let mut item2 = Item::default();
        item2.set_title("2".to_string());
        item2.set_pub_date("Tue, 06 Oct 2020 17:00:53 GMT".to_string());

        let feed_items = vec![item1, item2];

        let channel = ChannelBuilder::default()
            .title("dummy channel".to_string())
            .items(feed_items)
            .build();

        let channel_with_time = RssResponse {
            channel,
            time_to_fetch: Duration::new(2, 0),
        };

        let actual = transform_feed_channel(channel_with_time).unwrap();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn fetch_feed_from_invalid_url() {
        let url: Url = Url::parse("https://www.google.com").unwrap();

        assert!(fetch_rss_feed(&url).await.is_err());
    }

    async fn mock_fetch_feeds() -> FeedChannel {
        let items = vec![
            FeedItem {
                title: String::from("1"),
                pub_date: NaiveDate::from_ymd_opt(2021, 01, 01).unwrap(),
            },
            FeedItem {
                title: String::from("2"),
                pub_date: NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
            },
            FeedItem {
                title: String::from("3"),
                pub_date: NaiveDate::from_ymd_opt(2019, 01, 01).unwrap(),
            },
            FeedItem {
                title: String::from("4"),
                pub_date: NaiveDate::from_ymd_opt(2018, 01, 01).unwrap(),
            },
        ];

        return FeedChannel {
            channel_name: String::from("mock channel"),
            time_to_fetch: Duration::new(2, 0),
            items,
        };
    }
}
