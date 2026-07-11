use walkers::{HttpTiles, Map, MapMemory, lon_lat, sources::OpenStreetMap};

fn main() -> eframe::Result {
    env_logger::init();
    eframe::run_native(
        "v2x-viz",
        Default::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc.egui_ctx.clone())))),
    )
}

struct App {
    tiles: HttpTiles,
    map_memory: MapMemory,
}

impl App {
    fn new(egui_ctx: egui::Context) -> Self {
        Self {
            tiles: HttpTiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.add(Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            lon_lat(17.03664, 51.09916),
        ));
    }
}
