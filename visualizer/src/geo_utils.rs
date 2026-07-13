/// Radius of the Earth in meters.
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculates the bounding box coordinates for a given center point and dimensions.
///
/// # Arguments
///
/// * `longitude` - Longitude of the point marking the center of the bounding box.
/// * `latitude` - Latitude of the point marking the center of the bounding box.
/// * `length_m` - Length of the bounding box in meters.
/// * `width_m` - Width of the bounding box in meters.
/// * `heading_deg` - Heading of the bounding box in degrees from True North.
///
/// # Returns
///
/// A tuple containing two coordinates `((top_left_lon, top_left_lat), (bottom_right_lon, bottom_right_lat))`.
pub fn get_bounding_box(
    longitude: f64,
    latitude: f64,
    length_m: f64,
    width_m: f64,
    heading_deg: f64,
) -> ((f64, f64), (f64, f64)) {
    let half_length = length_m / 2.0;
    let half_width = width_m / 2.0;

    let heading_deg = heading_deg.to_radians();
    let sin_h = heading_deg.sin();
    let cos_h = heading_deg.cos();

    // assume (0,0) is the center of the box
    // create corners before rotation
    let local_corners = [
        (-half_width, half_length),  // Top-Left
        (half_width, half_length),   // Top-Right
        (half_width, -half_length),  // Bottom-Right
        (-half_width, -half_length), // Bottom-Left
    ];

    // enforce minimum value to avoid division by 0
    let lat_cos = latitude.to_radians().cos().max(1e-10);

    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;

    // rotate each corner and convert to lat/lon
    for &(x, y) in &local_corners {
        let x_rot = x * cos_h + y * sin_h; // East/West displacement in meters.
        let y_rot = -x * sin_h + y * cos_h; // North/South displacement in meters.

        // Convert meter displacement to degree displacement
        let lat_disp = (y_rot / EARTH_RADIUS_M).to_degrees();
        let lon_disp = (x_rot / (EARTH_RADIUS_M * lat_cos)).to_degrees();

        let corner_lat = latitude + lat_disp;
        let corner_lon = longitude + lon_disp;

        min_lat = min_lat.min(corner_lat);
        max_lat = max_lat.max(corner_lat);
        min_lon = min_lon.min(corner_lon);
        max_lon = max_lon.max(corner_lon);
    }

    ((min_lon, max_lat), (max_lon, min_lat))
}
