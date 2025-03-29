pub fn xyz_to_coordinate(x: u32, y: u32, z: u16) -> (f64, f64) {
    // Size
    let n = 2_f64.powi(i32::from(z));

    // Longitude
    let longitude = ((f64::from(x) / n) * 360.0) - 180.0;

    // Latitude
    let latitude = (180.0 / std::f64::consts::PI)
        * (std::f64::consts::PI * (1.0 - 2.0 * (f64::from(y) / n)))
            .sinh()
            .atan();

    // Return
    (latitude, longitude)
}

pub fn coordinates_confine(
    lat: f64,
    lon: f64,
    top: f64,
    left: f64,
    bottom: f64,
    right: f64,
    size_x: u32,
    size_y: u32,
) -> (u32, u32) {
    // Get ratios
    let ratio_x: f64 = f64::from(size_x) / (left - right);
    let ratio_y: f64 = f64::from(size_y) / (top - bottom);

    // Offset items
    let item_x: f64 = (left - lon) * ratio_x;
    let item_y: f64 = (top - lat) * ratio_y;

    (item_x.floor() as u32, item_y.floor() as u32)
}

pub fn translate_edge(
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    ratio_x: f64,
    ratio_y: f64,
) -> (u32, u32) {
    // Offsets
    let offset_x: u32 = (f64::from(width) * ratio_x) as u32;
    let offset_y: u32 = (f64::from(height) * ratio_y) as u32;

    // Translate
    let translated_x: u32 = if x > offset_x { x - offset_x } else { 0 };
    let translated_y: u32 = if y > offset_y { y - offset_y } else { 0 };

    (translated_x, translated_y)
}
