mod cli;
mod sumo;
mod v2x;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use cli::Cli;
use sumo::{connect_with_retry, launch_sumo, run_simulation};
use v2x::build_publishers;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Cli::parse();
    let ctrlc_pressed = register_ctrlc_handler()?;

    let _sumo_process = if args.launch_sumo {
        Some(launch_sumo(&args.sumo, args.port)?)
    } else {
        None
    };

    let mut client = connect_with_retry(
        &args.host,
        args.port,
        Duration::from_secs(args.connect_timeout_secs),
    )?;

    let mut publishers = build_publishers(&args.messages);
    run_simulation(&mut client, &mut publishers, ctrlc_pressed.as_ref())
}

fn register_ctrlc_handler() -> anyhow::Result<Arc<AtomicBool>> {
    let ctrlc_requested = Arc::new(AtomicBool::new(false));
    let signal_flag = Arc::clone(&ctrlc_requested);

    ctrlc::set_handler(move || {
        signal_flag.store(true, Ordering::SeqCst);
    })
    .context("failed to install Ctrl-C handler")?;

    Ok(ctrlc_requested)
}
