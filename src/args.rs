use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "red-panda-chat")]
#[command(about = "A real-time chat application using Redpanda/Kafka")]
#[command(version)]
pub struct Args {
    /// Kafka/Redpanda bootstrap servers
    #[arg(short, long, default_value = "localhost:9092")]
    pub bootstrap_servers: String,

    /// Topic name for the chat room
    #[arg(short, long, default_value = "chat-room")]
    pub topic: String,

    /// Username for the chat
    #[arg(short, long)]
    pub username: Option<String>,

    /// Data format for messages
    #[arg(short = 'f', long, default_value = "text")]
    pub format: DataFormat,

    /// Admin: List all messages and exit
    #[arg(long)]
    pub list: bool,

    /// Admin: Update message at offset
    #[arg(long)]
    pub update: Option<i64>,

    /// Admin: New content for message update
    #[arg(long)]
    pub content: Option<String>,

    /// Admin: Delete message at offset
    #[arg(long)]
    pub delete: Option<i64>,

    /// Admin: Timeout in seconds for listing messages (default: 5)
    #[arg(long, default_value = "5")]
    pub timeout: u64,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum DataFormat {
    Text,
    Json,
    Protobuf,
}
