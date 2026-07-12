use anyhow::{Result, anyhow};
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
            VehicleLength, VehicleLengthConfidenceIndication, VehicleLengthValue, VehicleWidth,
            YawRate, YawRateConfidence, YawRateValue,
        },
    },
};

use crate::encoding::V2xEncoding;
use crate::station::ItsStationType;

/// CAM protocol version.
pub const CAM_PROTOCOL_VERSION: u8 = 2;

/// Message ID for Cooperative Awareness Message (CAM).
pub const CAM_MESSAGE_ID: u8 = 2;

/// Modulo value for ETSI ITS GenerationDeltaTime.
pub const GENERATION_DELTA_TIME_MODULO: u64 = 65_536;

/// ETSI ITS epoch: 2004-01-01T00:00:00 UTC, as Unix timestamp in milliseconds.
pub const ITS_EPOCH_UNIX_MS: u64 = 1_072_915_200_000;

/// Minimum speed in meters per second.
pub const MIN_SPEED_MPS: f32 = 0.0;

/// Maximum speed in meters per second.
pub const MAX_SPEED_MPS: f32 = 163.82;

/// Total degrees in a full circle.
pub const DEGREES_IN_CIRCLE: f32 = 360.0;

/// Minimum longitudinal acceleration in m/s2.
pub const MIN_ACCEL_MPSS: f32 = -16.0;

/// Maximum longitudinal acceleration in m/s2.
pub const MAX_ACCEL_MPSS: f32 = 16.0;

/// Default semi-major/minor axis length of the position confidence ellipse (100 cm / 1.0 m).
pub const DEFAULT_SEMI_AXIS_LENGTH_CM: u16 = 100;

/// Default heading value for the confidence ellipse orientation (0 degrees).
pub const DEFAULT_CONF_ELLIPSE_HEADING_DEG: u16 = 0;

/// Default altitude value (0 meters).
pub const DEFAULT_ALTITUDE_VALUE: i32 = 0;

/// Default heading confidence (1: equal to or less than 1 degree).
pub const DEFAULT_HEADING_CONFIDENCE: u8 = 1;

/// Default speed confidence (1: equal to or less than 1 m/s).
pub const DEFAULT_SPEED_CONFIDENCE: u8 = 1;

/// Default vehicle length value in decimetres (40: 4.0 meters).
pub const DEFAULT_VEHICLE_LENGTH_DECIMETRES: u16 = 40;

/// Default vehicle width value in decimetres (18: 1.8 meters).
pub const DEFAULT_VEHICLE_WIDTH_DECIMETRES: u8 = 18;

/// Default acceleration confidence (1: equal to or less than 0.1 m/s²).
pub const DEFAULT_ACCELERATION_CONFIDENCE: u8 = 1;

/// Default curvature value (0).
pub const DEFAULT_CURVATURE_VALUE: i16 = 0;

/// Default yaw rate value (0).
pub const DEFAULT_YAW_RATE_VALUE: i16 = 0;

/// Telemetry data for a vehicle or station used to build a Cooperative Awareness Message (CAM).
#[derive(Debug, Clone, Copy)]
pub struct CamTelemetry {
    /// Unique station identifier.
    pub station_id: u32,
    /// ETSI ITS station type classification.
    pub station_type: ItsStationType,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// WGS-84 latitude in degrees.
    pub latitude_deg: f64,
    /// WGS-84 longitude in degrees.
    pub longitude_deg: f64,
    /// Speed in m/s (clamped to valid range).
    pub speed_mps: f64,
    /// Heading in degrees (wrapped to 0..360).
    pub heading_deg: f64,
    /// Longitudinal acceleration in m/s2 (clamped to valid range).
    pub accel_mpss: f64,
}

/// Helper struct containing pre-converted/parsed CAM fields.
struct CamFields {
    station_id: u32,
    station_type: ItsStationType,
    generation_delta_time: GenerationDeltaTime,
    latitude: Latitude,
    longitude: Longitude,
    speed: SpeedValue,
    heading: HeadingValue,
    accel: LongitudinalAccelerationValue,
}

/// Build and encode an ETSI CAM into a byte payload.
///
/// # Arguments
///
/// * `telemetry` - Telemetry data for the vehicle or station.
/// * `encoding` - ASN.1 encoding rules to use.
pub fn build_and_encode_cam(telemetry: CamTelemetry, encoding: V2xEncoding) -> Result<Vec<u8>> {
    let CamTelemetry {
        station_id,
        station_type,
        timestamp_ms,
        latitude_deg,
        longitude_deg,
        speed_mps,
        heading_deg,
        accel_mpss,
    } = telemetry;

    // Convert primitives -> c-its-parser types
    let its_timestamp_ms = timestamp_ms.saturating_sub(ITS_EPOCH_UNIX_MS);
    let generation_delta_time =
        GenerationDeltaTime((its_timestamp_ms % GENERATION_DELTA_TIME_MODULO) as u16);
    let latitude = Latitude::from_deg(latitude_deg);
    let longitude = Longitude::from_deg(longitude_deg);
    let speed_val = SpeedValue::from_mps((speed_mps as f32).clamp(MIN_SPEED_MPS, MAX_SPEED_MPS))
        .map_err(|e| anyhow!("invalid speed: {e}"))?;
    let heading_val = HeadingValue::from_deg((heading_deg as f32).rem_euclid(DEGREES_IN_CIRCLE))
        .map_err(|e| anyhow!("invalid heading: {e}"))?;
    let accel_val = LongitudinalAccelerationValue::from_mpss(
        (accel_mpss as f32).clamp(MIN_ACCEL_MPSS, MAX_ACCEL_MPSS),
    )
    .map_err(|e| anyhow!("invalid acceleration: {e}"))?;

    // Build CAM PDU
    let cam = build_cam(CamFields {
        station_id,
        station_type,
        generation_delta_time,
        latitude,
        longitude,
        speed: speed_val,
        heading: heading_val,
        accel: accel_val,
    });

    // Encode
    let encoding_rules: EncodingRules = encoding.into();
    ItsMessage::Cam {
        geonetworking: None,
        transport: None,
        etsi: Box::new(cam),
    }
    .encode(encoding_rules)
    .map_err(|e| anyhow!("failed to encode CAM: {e}"))
}

/// Assemble the CAM PDU struct from c-its-parser types.
fn build_cam(fields: CamFields) -> CAM {
    let CamFields {
        station_id,
        station_type,
        generation_delta_time,
        latitude,
        longitude,
        speed,
        heading,
        accel,
    } = fields;

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
    let basic_container = BasicContainer::new(station_type.into(), reference_position);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_encode_cam_with_unix_timestamp() {
        let telemetry = CamTelemetry {
            station_id: 42,
            station_type: ItsStationType::PassengerCar,
            // 2026-07-12 Unix timestamp in ms: roughly 1783864800000
            timestamp_ms: 1_783_864_800_000,
            latitude_deg: 48.137154,
            longitude_deg: 11.576124,
            speed_mps: 13.8,
            heading_deg: 180.0,
            accel_mpss: 0.5,
        };

        let result = build_and_encode_cam(telemetry, V2xEncoding::Uper);
        assert!(result.is_ok());

        // Test epoch boundary conversion
        // If we provide exactly ITS_EPOCH_UNIX_MS, its_timestamp_ms is 0.
        // If we provide ITS_EPOCH_UNIX_MS + 5000, its_timestamp_ms is 5000.
        let telemetry_epoch = CamTelemetry {
            station_id: 42,
            station_type: ItsStationType::PassengerCar,
            timestamp_ms: ITS_EPOCH_UNIX_MS + 5000,
            latitude_deg: 48.137154,
            longitude_deg: 11.576124,
            speed_mps: 13.8,
            heading_deg: 180.0,
            accel_mpss: 0.5,
        };
        let result_epoch = build_and_encode_cam(telemetry_epoch, V2xEncoding::Uper);
        assert!(result_epoch.is_ok());
    }
}
