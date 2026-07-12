use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use traci_rs::{SimulationScope, TraciClient};
use v2x::{
    cam::{CamTelemetry, build_and_encode_cam},
    encoding::V2xEncoding,
    station::ItsStationType,
};

use crate::cli::{CamEncoding, DestinationConfig, MessageType};

/// Publisher interface for V2X message generation per simulation step.
pub trait V2xPublisher {
    /// Publishes messages for all available vehicles in the current step.
    fn publish_step(&mut self, client: &mut TraciClient, sim_scope: &SimulationScope)
    -> Result<()>;
}

/// Builds one publisher per destination mapping, deduplicating identical entries.
pub fn build_publishers(destinations: &[DestinationConfig]) -> Result<Vec<Box<dyn V2xPublisher>>> {
    let mut publishers: Vec<Box<dyn V2xPublisher>> = Vec::new();
    let mut seen = HashSet::new();

    for destination in destinations {
        if !seen.insert(*destination) {
            continue;
        }

        match destination.message_type {
            MessageType::Cam => publishers.push(Box::new(EtsiCamPublisher::new(
                destination.socket,
                destination.encoding,
            )?)),
        }
    }

    Ok(publishers)
}

/// ETSI CAM publisher implementation.
struct EtsiCamPublisher {
    socket: UdpSocket,
    destination: SocketAddr,
    encoding: V2xEncoding,
    station_ids: HashMap<String, u32>,
    next_station_id: u32,
}

impl EtsiCamPublisher {
    fn new(destination: SocketAddr, encoding: CamEncoding) -> Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))
            .context("failed to bind UDP socket for CAM publisher")?;

        Ok(Self {
            socket,
            destination,
            encoding: encoding.into(),
            station_ids: HashMap::new(),
            next_station_id: 1,
        })
    }

    fn station_id_for(&mut self, vehicle_id: &str) -> u32 {
        if let Some(station_id) = self.station_ids.get(vehicle_id) {
            return *station_id;
        }

        let station_id = self.next_station_id;
        self.next_station_id = self.next_station_id.wrapping_add(1);
        self.station_ids.insert(vehicle_id.to_owned(), station_id);
        station_id
    }
}

impl V2xPublisher for EtsiCamPublisher {
    fn publish_step(
        &mut self,
        client: &mut TraciClient,
        sim_scope: &SimulationScope,
    ) -> Result<()> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let vehicle_ids = client.vehicle_get_id_list()?;

        for id in &vehicle_ids {
            let pos_2d = client.vehicle_get_position(id)?;
            let speed = client.vehicle_get_speed(id)?;
            let heading = client.vehicle_get_angle(id)?;
            let accel = client.vehicle_get_acceleration(id)?;

            let geo_pos = sim_scope.convert_geo(client, pos_2d.x, pos_2d.y, false)?;

            let station_id = self.station_id_for(id);

            let payload = build_and_encode_cam(
                CamTelemetry {
                    station_id,
                    station_type: ItsStationType::PassengerCar,
                    timestamp_ms,
                    latitude_deg: geo_pos.y,
                    longitude_deg: geo_pos.x,
                    speed_mps: speed,
                    heading_deg: heading,
                    accel_mpss: accel,
                },
                self.encoding,
            )
            .with_context(|| format!("failed to build CAM for vehicle '{id}'"))?;

            let bytes = self
                .socket
                .send_to(&payload, self.destination)
                .with_context(|| format!("failed to send CAM for vehicle '{id}' via UDP"))?;
            log::debug!("Sent {bytes} bytes to {}", self.destination);
        }

        Ok(())
    }
}
