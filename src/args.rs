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
}
