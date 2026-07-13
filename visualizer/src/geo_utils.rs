/// Radius of the Earth in meters.
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculates the rotated corner coordinates for a vehicle given a center point, dimensions, and heading.
///
/// # Arguments
///
/// * `longitude` - Longitude of the vehicle's reference position (front bumper center).
/// * `latitude` - Latitude of the vehicle's reference position (front bumper center).
/// * `length_m` - Length of the vehicle in meters.
/// * `width_m` - Width of the vehicle in meters.
/// * `heading_deg` - Heading of the vehicle in degrees from True North.
///
/// # Returns
///
/// An array containing four `(longitude, latitude)` tuples representing the rotated corners of the vehicle.
pub fn get_corners(
    longitude: f64,
    latitude: f64,
    length_m: f64,
    width_m: f64,
    heading_deg: f64,
) -> [(f64, f64); 4] {
    let half_width = width_m / 2.0;

    let heading_rad = heading_deg.to_radians();
    let sin_h = heading_rad.sin();
    let cos_h = heading_rad.cos();

    // Reference position (0,0) is the center of the vehicle's front bumper.
    let local_corners = [
        (-half_width, 0.0),       // Top-Left (Front-Left)
        (half_width, 0.0),        // Top-Right (Front-Right)
        (half_width, -length_m),  // Bottom-Right (Rear-Right)
        (-half_width, -length_m), // Bottom-Left (Rear-Left)
    ];

    // enforce minimum value to avoid division by 0
    let lat_cos = latitude.to_radians().cos().abs().max(1e-10);

    let mut corners = [(0.0, 0.0); 4];

    // rotate each corner and convert to lat/lon
    for (i, &(x, y)) in local_corners.iter().enumerate() {
        let x_disp = x * cos_h + y * sin_h; // East/West displacement in meters.
        let y_disp = -x * sin_h + y * cos_h; // North/South displacement in meters.

        // Convert meter displacement to degree displacement
        let lat_disp = (y_disp / EARTH_RADIUS_M).to_degrees();
        let lon_disp = (x_disp / (EARTH_RADIUS_M * lat_cos)).to_degrees();

        let corner_lat = latitude + lat_disp;
        let corner_lon = longitude + lon_disp;

        corners[i] = (corner_lon, corner_lat);
    }

    corners
}
