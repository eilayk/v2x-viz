use clap::{ArgAction, Args, Parser, ValueEnum};

/// Supported V2X message families to publish.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ValueEnum)]
pub enum MessageType {
    /// ETSI Cooperative Awareness Message (CAM).
    Cam,
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
#[command(about = "Runs SUMO and publishes V2X messages over TraCI")]
pub struct Cli {
    /// TraCI host.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// TraCI port.
    #[arg(long, default_value_t = 8813)]
    pub port: u16,
    /// Whether this process should launch SUMO itself.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub launch_sumo: bool,
    /// SUMO launch-specific options.
    #[command(flatten)]
    pub sumo: SumoLaunchOptions,
    /// Timeout for establishing the TraCI connection.
    #[arg(long, default_value_t = 15)]
    pub connect_timeout_secs: u64,
    /// Comma-separated message types to publish.
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = [MessageType::Cam]
    )]
    pub messages: Vec<MessageType>,
}
