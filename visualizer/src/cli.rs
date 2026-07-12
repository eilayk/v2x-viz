use clap::Parser;
use std::net::SocketAddr;

/// CLI arguments for the visualizer.
#[derive(Debug, Parser)]
#[command(about = "Visualizes V2X messages received over UDP")]
pub struct Cli {
    /// UDP socket address to listen on for incoming V2X packets.
    #[arg(long, default_value = "127.0.0.1:5000")]
    pub listen: SocketAddr,

    /// Initial map longitude.
    #[arg(long, default_value_t = -79.38433)]
    pub longitude: f64,

    /// Initial map latitude.
    #[arg(long, default_value_t = 43.65734)]
    pub latitude: f64,

    /// Initial map zoom level.
    #[arg(long, default_value_t = 18.0)]
    pub zoom: f64,
}
