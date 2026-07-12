/// Direction of travel reported in a CAM high-frequency container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VehicleDriveDirection {
    /// Vehicle is moving forward.
    Forward,
    /// Vehicle is moving in reverse.
    Backward,
    /// Direction is not available.
    Unavailable,
}

/// Decoded representation of a single vehicle, extracted from a CAM payload.
#[derive(Debug, Clone)]
pub struct DecodedCar {
    /// Unique ETSI ITS station identifier.
    pub station_id: u32,
    /// Raw ETSI station-type code (e.g. `5` = passenger car).
    pub station_type: u8,
    /// WGS-84 latitude in degrees.
    pub latitude_deg: f64,
    /// WGS-84 longitude in degrees.
    pub longitude_deg: f64,
    /// Altitude above the WGS-84 ellipsoid in metres, or `None` if unavailable.
    pub altitude_m: Option<f32>,
    /// Speed in m/s, or `None` if unavailable.
    pub speed_mps: Option<f32>,
    /// Heading measured clockwise from true north in degrees [0, 360), or `None` if unavailable.
    pub heading_deg: Option<f32>,
    /// Longitudinal acceleration in m/s2, or `None` if unavailable.
    pub accel_mpss: Option<f32>,
    /// Yaw rate in degrees per second, or `None` if unavailable.
    pub yaw_rate_deg_s: Option<f32>,
    /// Vehicle length in metres, or `None` if unavailable.
    pub vehicle_length_m: Option<f32>,
    /// Vehicle width in metres, or `None` if unavailable.
    pub vehicle_width_m: Option<f32>,
    /// Drive direction (forward / backward / unavailable).
    pub drive_direction: VehicleDriveDirection,
}

/// Top-level decoded V2X message.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum V2xMessage {
    /// A vehicle broadcasting its position and kinematics via a CAM.
    Car(DecodedCar),
}
