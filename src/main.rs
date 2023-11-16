use anyhow::Result;
use chrono::{DateTime, Utc};
use feed_rs::{model::Entry, model::Feed, parser};
use log::{error, info};
use std::{fs::File, io::Read, process::exit};
use webhook::client::{WebhookClient, WebhookResult};

const PROFILE_IMAGE_URL: &str = "https://i.pinimg.com/736x/c7/b8/11/c7b8113247fecd83bd9b5ed5bd3f34d5.jpg";
const CREATOR_IMAGE_URL: &str = "https://cdn.discordapp.com/avatars/297490782397399040/ff6fc9810917f2f41e72682c8fa02e4f?size=256";
const KERNEL_NEWS_RSS: &str = "https://www.kernel.org/feeds/kdist.xml";
const KERNEL_NEWS_RSS_LOCAL: &str = "/opt/kernel.xml";

#[tokio::main]
async fn main() -> WebhookResult<()> {
    env_logger::init();

    let env_url = std::env::var("KNEWSNOTIFD_WEBHOOK_URL");
    let webhook_url = if env_url.is_ok() {
        env_url.unwrap()
    } else {
        error!("KNEWSNOTIFD_WEBHOOK_URL is not set... No webhook URL was provided, exiting...");
        exit(1);
    };

    info!("knewsnotifd {} by Luxzi", env!("CARGO_PKG_VERSION"));
    info!("Attempting to access webhook at '{webhook_url}'");

    let client = WebhookClient::new(&webhook_url);
    let webhook_info = match client.get_information().await {
        Ok(o) => o,
        Err(e) => {
            error!("Could not contact webhook at '{webhook_url}'; {e}");
            exit(1);
        }
    };

    let mut last_post_time: Option<DateTime<Utc>> = None;

    info!("Webhook accessed successfully: {:?}", webhook_info);

    let task_interval = chrono::Duration::hours(2).to_std()?;
    let mut interval_timer = tokio::time::interval(task_interval);

    client
        .send(|message| {
            message
                .username("knewsnotifd")
                .avatar_url(PROFILE_IMAGE_URL)
                .embed(|embed| {
                    embed
                        .title(&format!(
                            "🟢 knewsnotifd {} online",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .footer("Made with ❤️ by luxzi", Some(CREATOR_IMAGE_URL.to_string()))
                })
        })
        .await?;

    loop {
        info!("Resyncing RSS feed in {task_interval:?}");
        interval_timer.tick().await;
        info!("Syncing RSS feed...");

        match sync_rss_feed().await {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to sync RSS feed... {e}");
            }
        };

        let feed: Option<Feed> = match parse_rss_feed().await {
            Ok(o) => Some(o),
            Err(e) => {
                error!("Failed to parse local RSS feed.. {e}");
                None
            }
        };

        if feed.is_some() {
            let feed = feed.unwrap();

            info!("RSS feed synced successfully");

            if last_post_time.is_some() && last_post_time != feed.updated {
                info!("Post list was updated...");
                post_to_channel(&client, &feed, last_post_time).await?;
            }

            last_post_time = feed.updated;
        }
    }
}

async fn sync_rss_feed() -> Result<()> {
    let resp = reqwest::get(KERNEL_NEWS_RSS).await?;
    let mut content = std::io::Cursor::new(resp.bytes().await?);
    let mut file = File::create(KERNEL_NEWS_RSS_LOCAL)?;
    std::io::copy(&mut content, &mut file)?;

    Ok(())
}

async fn parse_rss_feed() -> Result<Feed> {
    let mut local_rss = File::open(KERNEL_NEWS_RSS_LOCAL)?;
    let mut rss_feed: String = String::new();
    local_rss.read_to_string(&mut rss_feed)?;

    let feed = parser::parse(rss_feed.as_bytes())?;

    Ok(feed)
}

async fn post_to_channel(
    client: &WebhookClient,
    feed: &Feed,
    last_post_time: Option<DateTime<Utc>>,
) -> WebhookResult<()> {
    let mut posts: Vec<&Entry> = vec![];

    for post in &feed.entries {
        if post.published > last_post_time {
            posts.push(post);
        }
    }

    info!("Found {} new post(s)", posts.len());
    info!("Sending to webhook channel...");

    for post in posts {
        let title = &post.title.as_ref().unwrap().content;
        let desc = &post.summary.as_ref().unwrap().content;
        let changelog_url_grabber = desc.split_once("ChangeLog:");
        let changelog_url: Option<&str> = if changelog_url_grabber.is_some() {
            Some(
                changelog_url_grabber
                    .unwrap()
                    .1
                    .split('"')
                    .collect::<Vec<&str>>()[1],
            )
        } else {
            None
        };

        let changelog = if changelog_url.is_some() {
            format!("[View changelog]({})", changelog_url.unwrap())
        } else {
            "No change log".to_string()
        };

        client
            .send(|message| {
                message
                    .username("knewsnotifd")
                    .avatar_url(PROFILE_IMAGE_URL)
                    .embed(|embed| {
                        embed
                            .title(title)
                            .author("kernel.org", Some("https://kernel.org/".to_string()), None)
                            .description(&changelog)
                            .footer("Made with ❤️ by luxzi", Some(CREATOR_IMAGE_URL.to_string()))
                    })
            })
            .await?;

        tokio::time::sleep(chrono::Duration::seconds(10).to_std()?).await;
    }

    Ok(())
}
