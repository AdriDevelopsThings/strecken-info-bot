use std::f64::consts::PI;

// translate epsg 4326 coordinates to epsg 3857 coordinates
// x is lon, y is lat
pub(super) fn epsg_4326_to_epsg_3857(mut x: f64, mut y: f64) -> (f64, f64) {
    // source: https://developers.auravant.com/en/blog/2022/09/09/post-3/
    x = (x * 20037508.34) / 180f64;
    y = f64::tan(((90f64 + y) * PI) / 360f64).ln() / (PI / 180f64);
    y = (y * 20037508.34) / 180f64;
    (x, y)
}
