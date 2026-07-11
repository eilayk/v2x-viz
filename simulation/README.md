# Simulation

This Rust module runs a SUMO scenario, connects to SUMO via TraCI, processes simulated vehicles, and publishes ETSI CAM (Cooperative Awareness Message) outputs.

## Requirements

- **SUMO must be installed** and available in your `PATH` (`sumo` and/or `sumo-gui`).
- A convenient install option is:

```bash
uv tool install eclipse-sumo
```

- Rust/Cargo (for building and running this crate).

## Current scope

- **V2X messages:** only ETSI messages are supported right now (currently `CAM`), but the publisher architecture is designed to be extended with additional message families in the future.
- **Vehicle classes:** publishing is currently focused on cars; simulation support can be extended to other classes later.

## Run examples

From the repository root:

```bash
cargo run -p simulation -- \
  --scenario simulation/scenarios/toronto/osm.sumocfg
```

Headless SUMO:

```bash
cargo run -p simulation -- \
  --sumo-binary sumo \
  --scenario simulation/scenarios/toronto/osm.sumocfg
```

Connect to an already-running SUMO instance (do not auto-launch SUMO from Rust):

```bash
sumo -c simulation/scenarios/toronto/osm.sumocfg --remote-port 8813 --step-length 0.1 --start
```

```bash
cargo run -p simulation -- \
  --launch-sumo false \
  --host 127.0.0.1 \
  --port 8813
```
