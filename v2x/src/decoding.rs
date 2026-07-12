use anyhow::{Result, anyhow};
use c_its_parser::{
    Headers, ItsMessage,
    standards::cam_1_4_1::cam_pdu_descriptions::{CAM, HighFrequencyContainer},
    standards::cdd_1_3_1_1::its_container::DriveDirection,
};

use crate::decoded::{DecodedCar, V2xMessage, VehicleDriveDirection};

/// Decode a raw V2X byte payload into a typed [`V2xMessage`].
///
/// # Errors
///
/// Returns an error if the bytes cannot be decoded as a known V2X message type,
/// or if a required field is missing or malformed.
pub fn decode_v2x(bytes: &[u8]) -> Result<V2xMessage> {
    let its_message =
        c_its_parser::de::decode(bytes, Headers::None).map_err(|e| anyhow!("decode error: {e}"))?;

    match its_message {
        ItsMessage::Cam { etsi, .. } => match extract_from_cam(&etsi)? {
            CamVariant::Car(car) => Ok(V2xMessage::Car(car)),
            CamVariant::Unsupported => Err(anyhow!(
                "received CAM with unsupported high-frequency container"
            )),
        },
        #[allow(unreachable_patterns)]
        _ => Err(anyhow!(
            "unsupported V2X message type; only CAM is currently decoded"
        )),
    }
}

enum CamVariant {
    Car(DecodedCar),
    Unsupported,
}

/// Extract a [`CamVariant`] from a fully decoded CAM PDU struct.
fn extract_from_cam(cam: &CAM) -> Result<CamVariant> {
    let params = &cam.cam.cam_parameters;
    let basic = &params.basic_container;
    let ref_pos = &basic.reference_position;

    let station_id = cam.header.station_id.0;
    let station_type = basic.station_type.0;

    let latitude_deg = ref_pos.latitude.as_deg();
    let longitude_deg = ref_pos.longitude.as_deg();
    let altitude_m = ref_pos.altitude.altitude_value.try_as_meters();

    match &params.high_frequency_container {
        HighFrequencyContainer::basicVehicleContainerHighFrequency(hf) => {
            let speed_mps = hf.speed.speed_value.try_as_mps();
            let heading_deg = hf.heading.heading_value.try_as_deg();
            let accel_mpss = hf
                .longitudinal_acceleration
                .longitudinal_acceleration_value
                .try_as_mpss();
            let yaw_rate_deg_s = hf.yaw_rate.yaw_rate_value.try_as_deg_rate();

            let vehicle_length_m = hf.vehicle_length.vehicle_length_value.try_as_meters();
            let vehicle_width_m = hf.vehicle_width.try_as_meters();

            let drive_direction = match hf.drive_direction {
                DriveDirection::forward => VehicleDriveDirection::Forward,
                DriveDirection::backward => VehicleDriveDirection::Backward,
                DriveDirection::unavailable => VehicleDriveDirection::Unavailable,
            };

            Ok(CamVariant::Car(DecodedCar {
                station_id,
                station_type,
                latitude_deg,
                longitude_deg,
                altitude_m,
                speed_mps,
                heading_deg,
                accel_mpss,
                yaw_rate_deg_s,
                vehicle_length_m,
                vehicle_width_m,
                drive_direction,
            }))
        }
        _ => Ok(CamVariant::Unsupported),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cam::{CamTelemetry, ITS_EPOCH_UNIX_MS, build_and_encode_cam};
    use crate::encoding::V2xEncoding;
    use crate::station::ItsStationType;

    /// Build a CAM from known telemetry, decode it back and verify all fields
    /// round-trip correctly (within floating-point precision introduced by the
    /// ETSI fixed-point encoding).
    #[test]
    fn round_trip_uper() {
        let telemetry = CamTelemetry {
            station_id: 7,
            station_type: ItsStationType::PassengerCar,
            timestamp_ms: ITS_EPOCH_UNIX_MS + 10_000,
            latitude_deg: 48.137_154,
            longitude_deg: 11.576_124,
            speed_mps: 13.8,
            heading_deg: 270.0,
            accel_mpss: -1.5,
        };

        let bytes =
            build_and_encode_cam(telemetry, V2xEncoding::Uper).expect("encoding should succeed");

        let decoded = decode_v2x(&bytes).expect("decoding should succeed");

        let V2xMessage::Car(car) = decoded;

        assert_eq!(car.station_id, 7);
        assert_eq!(car.station_type, ItsStationType::PassengerCar as u8);

        assert_eq!(car.latitude_deg, telemetry.latitude_deg);
        assert_eq!(car.longitude_deg, telemetry.longitude_deg);

        assert_eq!(car.speed_mps, Some(telemetry.speed_mps as f32));
        assert_eq!(car.heading_deg, Some(telemetry.heading_deg as f32));
        assert_eq!(car.accel_mpss, Some(telemetry.accel_mpss as f32));

        assert_eq!(car.drive_direction, VehicleDriveDirection::Forward);
    }
}
