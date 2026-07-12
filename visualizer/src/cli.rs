use clap::Parser;
use std::net::SocketAddr;

/// CLI arguments for the visualizer.
#[derive(Debug, Parser)]
#[command(about = "Visualizes V2X messages received over UDP")]
pub struct Cli {
    /// UDP socket address to listen on for incoming V2X packets.
    #[arg(long, default_value = "127.0.0.1:5000")]
    pub listen: SocketAddr,
}
