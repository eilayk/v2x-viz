use clap::Parser;
use std::sync::mpsc::Receiver;
use walkers::{HttpTiles, Map, MapMemory, lon_lat, sources::OpenStreetMap};

mod cli;
mod udp_receiver;

fn main() -> eframe::Result {
    env_logger::init();

    let args = cli::Cli::parse();
    let udp_rx = udp_receiver::spawn(args.listen);

    eframe::run_native(
        "v2x-viz",
        Default::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc.egui_ctx.clone(), udp_rx)))),
    )
}

struct App {
    tiles: HttpTiles,
    map_memory: MapMemory,
    udp_rx: Receiver<udp_receiver::Packet>,
}

impl App {
    fn new(egui_ctx: egui::Context, udp_rx: Receiver<udp_receiver::Packet>) -> Self {
        Self {
            tiles: HttpTiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
            udp_rx,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Drain any packets that arrived since the last frame.
        while let Ok(packet) = self.udp_rx.try_recv() {
            log::debug!(
                "received {} byte(s) from {}",
                packet.data.len(),
                packet.source
            );
        }

        ui.add(Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            lon_lat(17.03664, 51.09916),
        ));

        ui.ctx().request_repaint();
    }
}
