/// Approximate meters per degree of latitude on WGS-84.
const METERS_PER_DEG_LAT: f64 = 111_320.0;

/// Calculates the bounding box coordinates for a given center point and dimensions.
///
/// # Arguments
///
/// * `longitude` - Longitude of the point marking the center of the bounding box.
/// * `latitude` - Latitude of the point marking the center of the bounding box.
/// * `length_m` - Length of the bounding box in meters.
/// * `width_m` - Width of the bounding box in meters.
///
/// # Returns
///
/// A tuple containing two coordinates `((top_left_lon, top_left_lat), (bottom_right_lon, bottom_right_lat))`.
pub fn get_bounding_box(
    longitude: f64,
    latitude: f64,
    length_m: f64,
    width_m: f64,
) -> ((f64, f64), (f64, f64)) {
    let half_length = length_m / 2.0;
    let half_width = width_m / 2.0;

    // enforce minimum value to avoid division by 0
    let lat_cos = latitude.to_radians().cos().max(1e-10);

    let lat_displacement = half_length / METERS_PER_DEG_LAT;
    let lon_displacement = half_width / (METERS_PER_DEG_LAT * lat_cos);

    let top_left_corner_lat = latitude + lat_displacement;
    let top_left_corner_lon = longitude - lon_displacement;
    let bottom_right_corner_lat = latitude - lat_displacement;
    let bottom_right_corner_lon = longitude + lon_displacement;

    (
        (top_left_corner_lon, top_left_corner_lat),
        (bottom_right_corner_lon, bottom_right_corner_lat),
    )
}
