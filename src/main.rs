mod args;
mod message;

use args::Args;
use clap::Parser;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer, DefaultConsumerContext};
use rdkafka::message::Message as KafkaMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::io;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use futures_util::StreamExt;
use anyhow::Result;
use message::MessageHandler;

/// Admin function: List all messages
async fn admin_list_messages(bootstrap_servers: &str, topic: &str, data_format: args::DataFormat, timeout_seconds: u64) -> Result<()> {
    let consumer: rdkafka::consumer::BaseConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", "admin-list")
        .set("bootstrap.servers", bootstrap_servers)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "10000")
        .set("heartbeat.interval.ms", "3000")
        .create()?;

    consumer.subscribe(&[topic])?;

    println!("📋 Listing all messages in topic '{}':", topic);
    println!("{}", "=".repeat(80));
    println!("🔍 Consumer subscribed, waiting for messages...");
    println!("🔧 Group ID: admin-list, Bootstrap: {}", bootstrap_servers);

    // Wait a bit for subscription to be ready
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let mut message_count = 0;
    let timeout = std::time::Duration::from_secs(timeout_seconds);
    let start_time = std::time::Instant::now();

    loop {
        if start_time.elapsed() > timeout {
            println!("⏰ Timeout reached after {} seconds. Found {} messages.", timeout_seconds, message_count);
            break;
        }

        match consumer.poll(std::time::Duration::from_millis(100)) {
            Some(Ok(msg)) => {
                println!("📨 Received message at offset: {}", msg.offset());
                if let Some(payload) = msg.payload() {
                    match MessageHandler::deserialize_message(payload, data_format) {
                        Ok((username, message)) => {
                            let timestamp = match msg.timestamp() {
                                rdkafka::Timestamp::CreateTime(ts) => ts,
                                rdkafka::Timestamp::LogAppendTime(ts) => ts,
                                rdkafka::Timestamp::NotAvailable => 0,
                            };
                            let actual_offset = msg.offset();
                            let replaces_info = if data_format == args::DataFormat::Json {
                                // For JSON, we need to parse the message to get replaces_offset
                                if let Ok(json_msg) = serde_json::from_str::<message::JsonMessage>(&message) {
                                    if let Some(replaces) = json_msg.replaces_offset {
                                        format!(" (replaces offset {})", replaces)
                                    } else {
                                        "".to_string()
                                    }
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string() // For now, only JSON shows this info
                            };
                            
                            println!("[{}] Offset: {}, User: {}, Message: {}{}", 
                                chrono::DateTime::from_timestamp_millis(timestamp).unwrap_or_default()
                                    .format("%Y-%m-%d %H:%M:%S"),
                                actual_offset, username, message, replaces_info);
                            message_count += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize message at offset {}: {}", msg.offset(), e);
                        }
                    }
                } else {
                    println!("[Tombstone] Offset: {}", msg.offset());
                    message_count += 1;
                }
            }
            Some(Err(e)) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
            None => {
                // No more messages for now, but continue polling until timeout
                continue;
            }
        }
    }
    
    if message_count == 0 {
        println!("ℹ️  No messages found in topic '{}'", topic);
    }
    
    Ok(())
}

/// Admin function: Update a message
async fn admin_update_message(
    bootstrap_servers: &str, 
    username: &str,
    topic: &str, 
    offset: i64, 
    content: &str, 
    data_format: args::DataFormat
) -> Result<()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .create()?;

    let serialized_data = MessageHandler::serialize_message(username, content, data_format, Some(offset), None)?;
    let record = FutureRecord::to(topic)
        .payload(&serialized_data)
        .key(b"admin-update");

    match producer.send(record, None).await {
        Ok(_) => {
            println!("✅ Updated message sent successfully!");
            println!("   Target offset: {}", offset);
            println!("   New content: {}: {}", username, content);
        }
        Err((e, _)) => {
            eprintln!("❌ Failed to send updated message: {}", e);
        }
    }

    Ok(())
}

/// Admin function: Delete a message (send deletion message)
async fn admin_delete_message(bootstrap_servers: &str, topic: &str, offset: i64, data_format: args::DataFormat) -> Result<()> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .create()?;

    let serialized_data = MessageHandler::serialize_message("Admin", "Message deleted", data_format, None, Some(offset))?;
    let record = FutureRecord::to(topic)
        .payload(&serialized_data)
        .key(b"admin-delete");

    match producer.send(record, None).await {
        Ok(_) => {
            println!("🗑️  Delete message sent for offset: {}", offset);
        }
        Err((e, _)) => {
            eprintln!("❌ Failed to send delete message: {}", e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Get configuration from args
    let bootstrap_servers = &args.bootstrap_servers;
    let topic = &args.topic;
    let data_format = args.format;

    // Handle admin operations first
    if args.list {
        return admin_list_messages(bootstrap_servers, topic, data_format, args.timeout).await;
    }
    // Regular chat mode - username is required
    let username = match args.username {
        Some(username) => username,
        None => {
            eprintln!("Error: Username is required for chat mode. Please provide it using the --username (-u) option.");
            return Err(anyhow::anyhow!("Username is required"));
        }
    };

    if let Some(offset) = args.update {
        let content = args.content.unwrap_or_else(|| "Message updated by admin".to_string());
        return admin_update_message(bootstrap_servers, &username, topic, offset, &content, data_format).await;
    }

    if let Some(offset) = args.delete {
        return admin_delete_message(bootstrap_servers, topic, offset, data_format).await;
    }

    let username = Arc::new(username);
    let group_id = format!("chat-group-{}", chrono::Utc::now().timestamp());

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
                            match MessageHandler::deserialize_message(payload, data_format) {
                                Ok((username, message)) => {
                                    // Format timestamp nicely
                                    let timestamp_str = match borrowed_message.timestamp() {
                                        rdkafka::Timestamp::CreateTime(ts) => {
                                            chrono::DateTime::from_timestamp_millis(ts).unwrap_or_default()
                                                .format("%H:%M:%S")
                                                .to_string()
                                        }
                                        rdkafka::Timestamp::LogAppendTime(ts) => {
                                            chrono::DateTime::from_timestamp_millis(ts).unwrap_or_default()
                                                .format("%H:%M:%S")
                                                .to_string()
                                        }
                                        rdkafka::Timestamp::NotAvailable => "N/A".to_string(),
                                    };
                                    println!("[{}] {}: {}", timestamp_str, username, message);
                                }
                                Err(e) => {
                                    eprintln!("Failed to deserialize message: {}", e);
                                }
                            }
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
                let message_text = line.unwrap();
                match MessageHandler::serialize_message(&username_clone, &message_text, data_format, None, None) {
                    Ok(serialized_data) => {
                        let record = FutureRecord::to(&topic_clone)
                            .payload(&serialized_data)
                            .key(b"key");
                        if let Err(e) = producer_clone.send(record, None).await {
                            eprintln!("Send error: {}", e.0);
                        }
                    }
                    Err(e) => {
                        eprintln!("Serialization error: {}", e);
                    }
                }
            }
        });
    });

    // Wait for threads
    consumer_handle.join()
        .and_then(|_| Ok(()))
        .or_else(|_| Err(anyhow::anyhow!("Consumer thread failed")))
}

