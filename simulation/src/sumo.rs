use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    process::{Child, Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail};
use traci_rs::{SimulationScope, TraciClient};

use crate::{cli::SumoLaunchOptions, v2x::V2xPublisher};

/// Handle for a child SUMO process that is cleaned up on drop.
pub struct ManagedSumoProcess {
    child: Child,
}

impl Drop for ManagedSumoProcess {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Launches SUMO with CLI-provided options and returns a managed process handle.
pub fn launch_sumo(args: &SumoLaunchOptions, port: u16) -> Result<ManagedSumoProcess> {
    let scenario = args
        .scenario
        .as_deref()
        .ok_or_else(|| anyhow!("--scenario is required when --launch-sumo=true"))?;
    let scenario_path = Path::new(scenario);
    if !scenario_path.exists() {
        bail!("Scenario file does not exist.")
    }

    let mut cmd = Command::new(args.sumo_binary.command_name());
    cmd.arg("-c")
        .arg(scenario_path)
        .arg("--remote-port")
        .arg(port.to_string())
        .arg("--step-length")
        .arg(args.step_length.to_string())
        .arg("--delay")
        .arg(args.delay_ms.to_string())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    if args.start {
        cmd.arg("--start");
    }

    let child = cmd
        .spawn()
        .with_context(|| format!("failed to launch {}", args.sumo_binary.command_name()))?;

    Ok(ManagedSumoProcess { child })
}

/// Connects to a TraCI server, retrying until timeout.
pub fn connect_with_retry(host: &str, port: u16, timeout: Duration) -> Result<TraciClient> {
    let started_at = Instant::now();

    loop {
        match TraciClient::connect(host, port) {
            Ok(client) => return Ok(client),
            Err(err) => {
                let last_error = err.to_string();
                if started_at.elapsed() >= timeout {
                    bail!(
                        "failed to connect to TraCI at {}:{} after {:?}: {}",
                        host,
                        port,
                        timeout,
                        last_error
                    );
                }
            }
        }

        sleep(Duration::from_millis(200));
    }
}

/// Runs SUMO simulation steps and forwards each step to all configured publishers.
pub fn run_simulation(
    client: &mut TraciClient,
    publishers: &mut [Box<dyn V2xPublisher>],
    shutdown_requested: &AtomicBool,
) -> Result<()> {
    let sim_scope = SimulationScope::default();
    let simulation_result = (|| -> Result<()> {
        while !shutdown_requested.load(Ordering::Relaxed) && client.simulation_step(0.0)? {
            let vehicle_ids = client.vehicle_get_id_list()?;
            for publisher in publishers.iter_mut() {
                publisher.publish_step(client, &sim_scope, &vehicle_ids)?;
            }
        }

        Ok(())
    })();

    let close_result = client.close();
    match (simulation_result, close_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(sim_err), Ok(())) => Err(sim_err),
        (Ok(()), Err(close_err)) => Err(close_err.into()),
        (Err(sim_err), Err(close_err)) => {
            Err(sim_err.context(format!("also failed to close TraCI client: {close_err}")))
        }
    }
}
