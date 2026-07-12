use clap::Parser;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use walkers::{HttpTiles, Map, MapMemory, lon_lat, sources::OpenStreetMap};

mod cli;
mod plugins;
mod udp_receiver;

fn main() -> eframe::Result {
    env_logger::init();

    let args = cli::Cli::parse();
    let udp_rx = udp_receiver::spawn(args.listen);
    let longitude = args.longitude;
    let latitude = args.latitude;
    let zoom = args.zoom;

    eframe::run_native(
        "v2x-viz",
        Default::default(),
        Box::new(move |cc| {
            Ok(Box::new(App::new(
                cc.egui_ctx.clone(),
                udp_rx,
                longitude,
                latitude,
                zoom,
            )))
        }),
    )
}

struct App {
    tiles: HttpTiles,
    map_memory: MapMemory,
    udp_rx: Receiver<udp_receiver::Packet>,
    objects: Arc<Vec<plugins::MapObject>>,
    longitude: f64,
    latitude: f64,
}

impl App {
    fn new(
        egui_ctx: egui::Context,
        udp_rx: Receiver<udp_receiver::Packet>,
        longitude: f64,
        latitude: f64,
        zoom: f64,
    ) -> Self {
        let mut map_memory = MapMemory::default();
        map_memory.set_zoom(zoom).ok();
        Self {
            tiles: HttpTiles::new(OpenStreetMap, egui_ctx),
            map_memory,
            udp_rx,
            objects: Arc::new(vec![]),
            longitude,
            latitude,
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

        ui.add(
            Map::new(
                Some(&mut self.tiles),
                &mut self.map_memory,
                lon_lat(self.longitude, self.latitude),
            )
            .with_plugin(plugins::ObjectsPlugin {
                objects: Arc::clone(&self.objects),
            }),
        );

        ui.ctx().request_repaint();
    }
}
