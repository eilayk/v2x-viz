use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, UdpSocket},
};

use anyhow::{Context, Result, anyhow};
use c_its_parser::{
    EncodingRules, ItsMessage,
    standards::{
        cam_1_4_1::cam_pdu_descriptions::{
            BasicContainer, BasicVehicleContainerHighFrequency, CAM, CamParameters, CoopAwareness,
            GenerationDeltaTime, HighFrequencyContainer,
        },
        cdd_1_3_1_1::its_container::{
            AccelerationConfidence, Altitude, AltitudeConfidence, AltitudeValue, Curvature,
            CurvatureCalculationMode, CurvatureConfidence, CurvatureValue, DriveDirection, Heading,
            HeadingConfidence, HeadingValue, ItsPduHeader, Latitude, Longitude,
            LongitudinalAcceleration, LongitudinalAccelerationValue, PosConfidenceEllipse,
            ReferencePosition, SemiAxisLength, Speed, SpeedConfidence, SpeedValue, StationID,
            StationType, TimestampIts, VehicleLength, VehicleLengthConfidenceIndication,
            VehicleLengthValue, VehicleWidth, YawRate, YawRateConfidence, YawRateValue,
        },
    },
};
use traci_rs::{SimulationScope, TraciClient};

use crate::cli::{CamEncoding, DestinationConfig, MessageType};

// ETSI ITS standard constants
const CAM_PROTOCOL_VERSION: u8 = 2;

/// Message ID for Cooperative Awareness Message (CAM).
const CAM_MESSAGE_ID: u8 = 2;

/// Modulo value for ETSI ITS GenerationDeltaTime.
const GENERATION_DELTA_TIME_MODULO: u64 = 65_536;

/// Minimum speed in meters per second.
const MIN_SPEED_MPS: f32 = 0.0;

/// Maximum speed in meters per second.
const MAX_SPEED_MPS: f32 = 163.82;

/// Total degrees in a full circle.
const DEGREES_IN_CIRCLE: f32 = 360.0;

/// Minimum longitudinal acceleration in m/s^2.
const MIN_ACCEL_MPSS: f32 = -16.0;

/// Maximum longitudinal acceleration in m/s^2.
const MAX_ACCEL_MPSS: f32 = 16.0;

/// Station type ID for a passenger car.
const STATION_TYPE_PASSENGER_CAR: u8 = 5;

/// Default semi-major/minor axis length of the position confidence ellipse (100 cm / 1.0 m).
const DEFAULT_SEMI_AXIS_LENGTH_CM: u16 = 100;

/// Default heading value for the confidence ellipse orientation (0 degrees).
const DEFAULT_CONF_ELLIPSE_HEADING_DEG: u16 = 0;

/// Default altitude value (0 meters).
const DEFAULT_ALTITUDE_VALUE: i32 = 0;

/// Default heading confidence (1: equal to or less than 1 degree).
const DEFAULT_HEADING_CONFIDENCE: u8 = 1;

/// Default speed confidence (1: equal to or less than 1 m/s).
const DEFAULT_SPEED_CONFIDENCE: u8 = 1;

/// Default vehicle length value in decimetres (40: 4.0 meters).
const DEFAULT_VEHICLE_LENGTH_DECIMETRES: u16 = 40;

/// Default vehicle width value in decimetres (18: 1.8 meters).
const DEFAULT_VEHICLE_WIDTH_DECIMETRES: u8 = 18;

/// Default acceleration confidence (1: equal to or less than 0.1 m/s^2).
const DEFAULT_ACCELERATION_CONFIDENCE: u8 = 1;

/// Default curvature value (0).
const DEFAULT_CURVATURE_VALUE: i16 = 0;

/// Default yaw rate value (0).
const DEFAULT_YAW_RATE_VALUE: i16 = 0;

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
    encoding_rules: EncodingRules,
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
            encoding_rules: to_encoding_rules(encoding),
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
        let now = chrono::Utc::now();
        let timestamp_its = TimestampIts::from(now);
        let generation_delta_time =
            GenerationDeltaTime((timestamp_its.0 % GENERATION_DELTA_TIME_MODULO) as u16);

        let vehicle_ids = client.vehicle_get_id_list()?;

        for id in &vehicle_ids {
            let pos_2d = client.vehicle_get_position(id)?;
            let speed = client.vehicle_get_speed(id)?;
            let heading = client.vehicle_get_angle(id)?;
            let accel = client.vehicle_get_acceleration(id)?;

            let geo_pos = sim_scope.convert_geo(client, pos_2d.x, pos_2d.y, false)?;

            let latitude = Latitude::from_deg(geo_pos.y);
            let longitude = Longitude::from_deg(geo_pos.x);
            let speed_val =
                SpeedValue::from_mps((speed as f32).clamp(MIN_SPEED_MPS, MAX_SPEED_MPS))
                    .map_err(|err| anyhow!("invalid speed: {err}"))?;
            let heading_val =
                HeadingValue::from_deg((heading as f32).rem_euclid(DEGREES_IN_CIRCLE))
                    .map_err(|err| anyhow!("invalid heading: {err}"))?;
            let accel_val = LongitudinalAccelerationValue::from_mpss(
                (accel as f32).clamp(MIN_ACCEL_MPSS, MAX_ACCEL_MPSS),
            )
            .map_err(|err| anyhow!("invalid acceleration: {err}"))?;
            let station_id = self.station_id_for(id);

            let cam = build_cam(
                station_id,
                generation_delta_time.clone(),
                latitude,
                longitude,
                speed_val,
                heading_val,
                accel_val,
            );

            let payload = ItsMessage::Cam {
                geonetworking: None,
                transport: None,
                etsi: Box::new(cam),
            }
            .encode(self.encoding_rules)
            .map_err(|err| anyhow!("failed to encode CAM for vehicle '{id}': {err}"))?;

            self.socket
                .send_to(&payload, self.destination)
                .with_context(|| format!("failed to send CAM for vehicle '{id}' via UDP"))?;
        }

        Ok(())
    }
}

fn to_encoding_rules(encoding: CamEncoding) -> EncodingRules {
    match encoding {
        CamEncoding::Uper => EncodingRules::UPER,
        CamEncoding::Xer => EncodingRules::XER,
        CamEncoding::Jer => EncodingRules::JER,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_cam(
    station_id: u32,
    generation_delta_time: GenerationDeltaTime,
    latitude: Latitude,
    longitude: Longitude,
    speed: SpeedValue,
    heading: HeadingValue,
    accel: LongitudinalAccelerationValue,
) -> CAM {
    let header = ItsPduHeader::new(CAM_PROTOCOL_VERSION, CAM_MESSAGE_ID, StationID(station_id));

    let reference_position = ReferencePosition::new(
        latitude,
        longitude,
        PosConfidenceEllipse::new(
            SemiAxisLength(DEFAULT_SEMI_AXIS_LENGTH_CM),
            SemiAxisLength(DEFAULT_SEMI_AXIS_LENGTH_CM),
            HeadingValue(DEFAULT_CONF_ELLIPSE_HEADING_DEG),
        ),
        Altitude::new(
            AltitudeValue(DEFAULT_ALTITUDE_VALUE),
            AltitudeConfidence::unavailable,
        ),
    );
    let basic_container =
        BasicContainer::new(StationType(STATION_TYPE_PASSENGER_CAR), reference_position);

    let high_frequency = BasicVehicleContainerHighFrequency::new(
        Heading::new(heading, HeadingConfidence(DEFAULT_HEADING_CONFIDENCE)),
        Speed::new(speed, SpeedConfidence(DEFAULT_SPEED_CONFIDENCE)),
        DriveDirection::forward,
        VehicleLength::new(
            VehicleLengthValue(DEFAULT_VEHICLE_LENGTH_DECIMETRES),
            VehicleLengthConfidenceIndication::noTrailerPresent,
        ),
        VehicleWidth(DEFAULT_VEHICLE_WIDTH_DECIMETRES),
        LongitudinalAcceleration::new(
            accel,
            AccelerationConfidence(DEFAULT_ACCELERATION_CONFIDENCE),
        ),
        Curvature::new(
            CurvatureValue(DEFAULT_CURVATURE_VALUE),
            CurvatureConfidence::unavailable,
        ),
        CurvatureCalculationMode::yawRateNotUsed,
        YawRate::new(
            YawRateValue(DEFAULT_YAW_RATE_VALUE),
            YawRateConfidence::unavailable,
        ),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let cam_parameters = CamParameters::new(
        basic_container,
        HighFrequencyContainer::basicVehicleContainerHighFrequency(high_frequency),
        None,
        None,
    );
    let cam = CoopAwareness::new(generation_delta_time, cam_parameters);

    CAM::new(header, cam)
}
