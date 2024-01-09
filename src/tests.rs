#[cfg(test)]

mod tests {
    use crate::{fetch_feed, filter_with, transform, FeedItem, FeedChannel};
    use chrono::NaiveDate;
    use rss::{ChannelBuilder, Item};

    #[tokio::test]
    async fn filter_items_with_date() {
        let mut expected = vec![FeedItem {
            title: String::from("1"),
            pub_date: NaiveDate::from_ymd_opt(2021, 01, 01).unwrap(),
        }];

        let feeds = mock_fetch_feeds().await;

        let mut filtered_feed = filter_with(
            &NaiveDate::from_ymd_opt(2020, 12, 30).unwrap(),
            feeds.clone(),
        )
        .await;

        assert!(filtered_feed.items.iter().eq(expected.iter()));

        expected = vec![];

        filtered_feed = filter_with(
            &NaiveDate::from_ymd_opt(2023, 12, 30).unwrap(),
            feeds.clone(),
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

        filtered_feed = filter_with(
            &NaiveDate::from_ymd_opt(2010, 12, 30).unwrap(),
            feeds.clone(),
        )
        .await;

        assert!(filtered_feed.items.iter().eq(expected.iter()));
    }

    #[tokio::test]
    async fn successful_channel_transformation() {
        let expected = FeedChannel {
            channel_name: "dummy channel".to_string(),
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

        let actual = transform(channel).await;

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    #[should_panic(expected = "InvalidStartTag")]
    async fn fetch_feed_from_invalid_url() {
        let url = "https://www.google.com";

        fetch_feed(url).await;
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
            items,
        };
    }
}
