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
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum DataFormat {
    Text,
    Json,
    Protobuf,
}

impl Args {
    #[allow(dead_code)]
    pub fn get_username(&self) -> String {
        if let Some(username) = &self.username {
            username.clone()
        } else {
            // If no username provided, ask for it interactively
            println!("Enter your username:");
            let mut username = String::new();
            std::io::stdin()
                .read_line(&mut username)
                .expect("Failed to read username");
            username.trim().to_string()
        }
    }
}
