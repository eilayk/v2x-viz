# Visualizer

An `egui` and `walkers` application to display V2X traffic on an OpenStreetMap map.

## How it works

- Listens for incoming V2X messages as **UDP packets**.
- Valid packets are decoded with the shared `v2x` library.
- Decoded messages are shown on the map when they contain supported vehicle data.

## CLI usage

From the repository root:

```bash
cargo run -p visualizer
```

Listen on a different UDP port:

```bash
cargo run -p visualizer -- --listen 0.0.0.0:5000
```

Start the map at a different location:

```bash
cargo run -p visualizer -- \
  --listen 127.0.0.1:5000 \
  --longitude -79.38433 \
  --latitude 43.65734 \
  --zoom 18
```

## Defaults

- `--listen 127.0.0.1:5000`
- `--longitude -79.38433`
- `--latitude 43.65734`
- `--zoom 18`
