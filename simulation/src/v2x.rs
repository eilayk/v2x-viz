use std::collections::HashSet;

use anyhow::Result;
use traci_rs::{SimulationScope, TraciClient};

use crate::cli::MessageType;

/// Publisher interface for V2X message generation per simulation step.
pub trait V2xPublisher {
    /// Publishes messages for all available vehicles in the current step.
    fn publish_step(
        &mut self,
        client: &mut TraciClient,
        sim_scope: &SimulationScope,
        vehicle_ids: &[String],
    ) -> Result<()>;
}

/// Builds one publisher per selected message type, deduplicating inputs.
pub fn build_publishers(message_types: &[MessageType]) -> Vec<Box<dyn V2xPublisher>> {
    let mut publishers: Vec<Box<dyn V2xPublisher>> = Vec::new();
    let mut seen = HashSet::new();

    for message in message_types {
        if !seen.insert(*message) {
            continue;
        }

        match message {
            MessageType::Cam => publishers.push(Box::new(EtsiCamPublisher)),
        }
    }

    publishers
}

/// ETSI CAM publisher implementation.
struct EtsiCamPublisher;

impl V2xPublisher for EtsiCamPublisher {
    fn publish_step(
        &mut self,
        client: &mut TraciClient,
        sim_scope: &SimulationScope,
        vehicle_ids: &[String],
    ) -> Result<()> {
        for id in vehicle_ids {
            let pos_2d = client.vehicle_get_position(id)?;
            let speed = client.vehicle_get_speed(id)?;
            let heading = client.vehicle_get_angle(id)?;
            let accel = client.vehicle_get_acceleration(id)?;

            let geo_pos = sim_scope.convert_geo(client, pos_2d.x, pos_2d.y, true)?;
            let lat_deg = geo_pos.y;
            let lon_deg = geo_pos.x;

            let lat_etsi = (lat_deg * 10_000_000.0).round() as i32;
            let lon_etsi = (lon_deg * 10_000_000.0).round() as i32;
            let speed_etsi = (speed * 100.0) as u16;
            let heading_etsi = (heading.rem_euclid(360.0) * 10.0).round() as u16;
            let accel_etsi = (accel * 10.0).round() as i16;

            println!(
                "CAM Veh {} | Lat: {}, Lon: {} | Speed: {} | Heading: {} | Accel: {} (ETSI)",
                id, lat_etsi, lon_etsi, speed_etsi, heading_etsi, accel_etsi
            );
        }

        Ok(())
    }
}
