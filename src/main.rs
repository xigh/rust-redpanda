mod args;

use args::Args;
use clap::Parser;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::io;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use futures_util::StreamExt;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Get configuration from args
    let bootstrap_servers = &args.bootstrap_servers;
    let topic = &args.topic;
    let group_id = format!("chat-group-{}", chrono::Utc::now().timestamp());  // Unique group ID to see all messages

    // Get username (from args or interactively)
    let username = match args.username {
        Some(username) => username,
        None => {
            eprintln!("Error: Username is required. Please provide it using the --username (-u) option.");
            return Err(anyhow::anyhow!("Username is required"));
        }
    };
    let username = Arc::new(username);

    // Create producer
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .create()
        .expect("Producer creation error");

    // Create consumer
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", bootstrap_servers)
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation error");
    consumer.subscribe(&[topic]).expect("Subscription error");

    // Thread to consume messages
    let consumer_handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut stream = consumer.stream();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(borrowed_message) => {
                        if let Some(payload) = borrowed_message.payload() {
                            let msg = String::from_utf8_lossy(payload);
                            println!("{}", msg);
                        }
                    }
                    Err(e) => println!("Consumption error: {}", e),
                }
            }
        });
    });

    // Main thread to produce messages from stdin
    let username_clone = username.clone();
    let producer_clone = producer.clone();
    let topic_clone = topic.to_string();
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let stdin = io::stdin();
            for line in stdin.lines() {
                let message = format!("{}: {}", *username_clone, line.unwrap());
                let record = FutureRecord::to(&topic_clone)
                    .payload(message.as_bytes())
                    .key(b"key");  // Optional key
                producer_clone.send(record, None).await.expect("Send error");
            }
        });
    });

    // Wait for threads
    consumer_handle.join()
        .and_then(|_| Ok(()))
        .or_else(|_| Err(anyhow::anyhow!("Consumer thread failed")))
}
