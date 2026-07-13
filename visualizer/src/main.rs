use clap::Parser;
use std::sync::mpsc::Receiver;
use std::{sync::Arc, time::Duration};
use walkers::{HttpTiles, Map, MapMemory, lon_lat, sources::OpenStreetMap};

mod cli;
mod geo_utils;
mod plugins;
mod tracked_objects;
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
    objects: tracked_objects::TrackedObjects<v2x::decoded::V2xMessage>,
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
            objects: tracked_objects::TrackedObjects::new(),
            longitude,
            latitude,
        }
    }

    /// Rebuild the plugin's object list from the current car state.
    fn build_objects(&self) -> Arc<Vec<plugins::MapObject>> {
        Arc::new(
            self.objects
                .values()
                .filter_map(|msg| match msg {
                    v2x::decoded::V2xMessage::Car(car) => car_to_map_object(car),
                    _ => None,
                })
                .collect(),
        )
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
                Ok(msg) => {
                    let station_id = match &msg {
                        v2x::decoded::V2xMessage::Car(car) => {
                            log::debug!(
                                "station {} at ({:.6}, {:.6})",
                                car.station_id,
                                car.latitude_deg,
                                car.longitude_deg,
                            );
                            Some(car.station_id)
                        }
                        _ => None,
                    };
                    if let Some(id) = station_id {
                        self.objects.insert(id, msg);
                    }
                }
                Err(e) => log::warn!("failed to decode V2X packet: {e}"),
            }
        }

        self.objects.clean_expired(Duration::from_millis(150));

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

    let corners = crate::geo_utils::get_corners(lon, lat, length_m, width_m, heading_deg);
    let points = corners
        .into_iter()
        .map(|(lon, lat)| lon_lat(lon, lat))
        .collect();

    Some(plugins::MapObject {
        shape: plugins::GeoShape::Polygon { points },
        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 100)),
        fill: egui::Color32::from_rgba_unmultiplied(0, 200, 100, 60),
    })
}
