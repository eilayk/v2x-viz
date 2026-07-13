use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use walkers::{HttpTiles, Map, MapMemory, lon_lat, sources::OpenStreetMap};

mod cli;
mod geo_utils;
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
    cars: HashMap<u32, v2x::decoded::DecodedCar>,
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
            cars: HashMap::new(),
            longitude,
            latitude,
        }
    }

    /// Rebuild the plugin's object list from the current car state.
    fn build_objects(&self) -> Arc<Vec<plugins::MapObject>> {
        Arc::new(self.cars.values().filter_map(car_to_map_object).collect())
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
            match v2x::decoding::decode_v2x(&packet.data) {
                Ok(v2x::decoded::V2xMessage::Car(car)) => {
                    log::debug!(
                        "station {} at ({:.6}, {:.6})",
                        car.station_id,
                        car.latitude_deg,
                        car.longitude_deg,
                    );
                    self.cars.insert(car.station_id, car);
                }
                Ok(_) => {}
                Err(e) => log::warn!("failed to decode V2X packet: {e}"),
            }
        }

        let objects = self.build_objects();

        ui.add(
            Map::new(
                Some(&mut self.tiles),
                &mut self.map_memory,
                lon_lat(self.longitude, self.latitude),
            )
            .with_plugin(plugins::ObjectsPlugin { objects }),
        );

        ui.ctx().request_repaint();
    }
}

/// Default length in meters used when the CAM omits vehicle size.
pub const DEFAULT_VEHICLE_LENGTH_M: f64 = 4.4;
/// Default width in meters used when the CAM omits vehicle size.
pub const DEFAULT_VEHICLE_WIDTH_M: f64 = 2.0;

/// Convert a decoded CAM car into a rectangular [`plugins::MapObject`].
///
/// Returns `None` if the position cannot be used.
fn car_to_map_object(car: &v2x::decoded::DecodedCar) -> Option<plugins::MapObject> {
    let lat = car.latitude_deg;
    let lon = car.longitude_deg;

    if !lat.is_finite() || !lon.is_finite() {
        return None;
    }

    let length_m = car
        .vehicle_length_m
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| v as f64)
        .unwrap_or(DEFAULT_VEHICLE_LENGTH_M);

    let width_m = car
        .vehicle_width_m
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| v as f64)
        .unwrap_or(DEFAULT_VEHICLE_WIDTH_M);

    let heading_deg = car
        .heading_deg
        .filter(|v| v.is_finite())
        .map(|v| v as f64)
        .unwrap_or(0.0);

    let ((top_left_lon, top_left_lat), (bottom_right_lon, bottom_right_lat)) =
        crate::geo_utils::get_bounding_box(lon, lat, length_m, width_m, heading_deg);

    let top_left = lon_lat(top_left_lon, top_left_lat);
    let bottom_right = lon_lat(bottom_right_lon, bottom_right_lat);

    Some(plugins::MapObject {
        shape: plugins::GeoShape::Rect {
            top_left,
            bottom_right,
        },
        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 100)),
        fill: egui::Color32::from_rgba_unmultiplied(0, 200, 100, 60),
    })
}
