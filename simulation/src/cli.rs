use clap::{ArgAction, Args, Parser, ValueEnum};
use std::{net::SocketAddr, str::FromStr};
use v2x::encoding::V2xEncoding;

/// Supported V2X message families to publish.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ValueEnum)]
pub enum MessageType {
    /// ETSI Cooperative Awareness Message (CAM).
    Cam,
}

/// Supported ASN.1 encoding rules for outgoing CAM payloads.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ValueEnum)]
pub enum CamEncoding {
    /// UPER.
    Uper,
    /// XML.
    Xer,
    /// JSON.
    Jer,
}

impl From<CamEncoding> for V2xEncoding {
    fn from(enc: CamEncoding) -> V2xEncoding {
        match enc {
            CamEncoding::Uper => V2xEncoding::Uper,
            CamEncoding::Xer => V2xEncoding::Xer,
            CamEncoding::Jer => V2xEncoding::Jer,
        }
    }
}

/// Destination mapping for one message type + encoding + socket target.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DestinationConfig {
    pub socket: SocketAddr,
    pub encoding: CamEncoding,
    pub message_type: MessageType,
}

impl FromStr for DestinationConfig {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.splitn(3, ':');
        let message_type_raw = parts.next().unwrap_or_default();
        let encoding_raw = parts.next().ok_or_else(|| {
            format!("invalid destination '{value}': expected format message:encoding:host:port")
        })?;
        let socket_raw = parts.next().ok_or_else(|| {
            format!("invalid destination '{value}': expected format message:encoding:host:port")
        })?;

        if message_type_raw.is_empty() || encoding_raw.is_empty() || socket_raw.is_empty() {
            return Err(format!(
                "invalid destination '{value}': expected format message:encoding:host:port"
            ));
        }

        let message_type = MessageType::from_str(message_type_raw, true).map_err(|_| {
            format!("invalid message type '{message_type_raw}' in destination '{value}'")
        })?;
        let encoding = CamEncoding::from_str(encoding_raw, true)
            .map_err(|_| format!("invalid encoding '{encoding_raw}' in destination '{value}'"))?;
        let socket = SocketAddr::from_str(socket_raw).map_err(|err| {
            format!("invalid socket '{socket_raw}' in destination '{value}': {err}")
        })?;

        Ok(Self {
            socket,
            encoding,
            message_type,
        })
    }
}

/// SUMO executable choice used when launching SUMO from this process.
#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum SumoBinary {
    /// Launch the headless `sumo` binary.
    Sumo,
    /// Launch the GUI-enabled `sumo-gui` binary.
    SumoGui,
}

impl SumoBinary {
    /// Returns the command name expected to be available in `PATH`.
    pub fn command_name(self) -> &'static str {
        match self {
            Self::Sumo => "sumo",
            Self::SumoGui => "sumo-gui",
        }
    }
}

/// SUMO launch-specific options.
#[derive(Debug, Clone, Args)]
pub struct SumoLaunchOptions {
    /// SUMO executable to launch when `--launch-sumo` is enabled.
    #[arg(long, value_enum, default_value_t = SumoBinary::SumoGui)]
    pub sumo_binary: SumoBinary,
    /// Path to SUMO `.sumocfg` scenario file (required when configured to launch SUMO).
    #[arg(long, required_if_eq("launch_sumo", "true"))]
    pub scenario: Option<String>,
    /// SUMO simulation step length in seconds.
    #[arg(long, default_value_t = 0.1)]
    pub step_length: f64,
    /// SUMO GUI delay in milliseconds.
    #[arg(long, default_value_t = 100)]
    pub delay_ms: u32,
    /// Pass `--start` to SUMO so the simulation starts immediately.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub start: bool,
}

/// CLI arguments for the simulation publisher.
#[derive(Debug, Parser)]
#[command(about = "Runs SUMO via TraCI and publishes encoded V2X messages over UDP")]
pub struct Cli {
    /// TraCI address.
    #[arg(long, default_value = "127.0.0.1:8813")]
    pub traci_addr: SocketAddr,
    /// Whether this process should launch SUMO itself.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub launch_sumo: bool,
    /// SUMO launch-specific options.
    #[command(flatten)]
    pub sumo: SumoLaunchOptions,
    /// Timeout for establishing the TraCI connection.
    #[arg(long, default_value_t = 15)]
    pub connect_timeout_secs: u64,
    /// Repeatable destination mapping in `message:encoding:host:port` format.
    ///
    /// Example:
    /// `--destination cam:uper:127.0.0.1:5000 --destination cam:jer:127.0.0.1:5001`
    #[arg(long = "destination", default_value = "cam:uper:127.0.0.1:5000")]
    pub destinations: Vec<DestinationConfig>,
}
