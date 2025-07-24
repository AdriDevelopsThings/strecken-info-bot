use std::f64::consts::{E, PI};

const EARTH_RADIUS_KM: f64 = 6371_f64;

// translate epsg 3857 coordinates to epsg 4326 coordinates
// first is lon, second is lat
pub fn epsg_3857_to_epsg_4326(mut x: f64, mut y: f64) -> (f64, f64) {
    // source: https://developers.auravant.com/en/blog/2022/09/09/post-3/
    x = (x * 180f64) / 20037508.34;
    y = (y * 180f64) / 20037508.34;
    y = (E.powf(y * (PI / 180f64)).atan() * 360f64) / PI - 90f64;
    (x, y)
}

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180f64
}

pub fn epsg_4326_distance_km(lon1: f64, mut lat1: f64, lon2: f64, mut lat2: f64) -> f64 {
    // source: https://stackoverflow.com/questions/365826/calculate-distance-between-2-gps-coordinates/365853#365853
    let d_lon = degrees_to_radians(lon2 - lon1);
    let d_lat = degrees_to_radians(lat2 - lat1);

    lat1 = degrees_to_radians(lat1);
    lat2 = degrees_to_radians(lat2);

    let a = (d_lat / 2f64).sin() * (d_lat / 2f64).sin()
        + (d_lon / 2f64).sin() * (d_lon / 2f64).sin() * lat1.cos() * lat2.cos();
    let c = 2f64 * a.sqrt().atan2((1f64 - a).sqrt());
    EARTH_RADIUS_KM * c
}
