/*************
 * Variables *
 *************/

// Tiling
pub const TILE_SIZE: usize = 256; // Pixels

// Icons
pub const ICON_RATIO_X: f64 = 0.5;
pub const ICON_RATIO_Y: f64 = 1.0;

/**************
 * Assignment *
 **************/

// Default
const DEFAULT: &str = "DEFAULT";

// Nothing
// pub const EMPTY: &[u8] = include_bytes!("../assets/empty.png");

// Jam
const ALERT_TYPE_JAM: &[&str; 3] = &[DEFAULT, "JAM_HEAVY_TRAFFIC", "JAM_STAND_STILL_TRAFFIC"];
const ALERT_TYPE_JAM_ASSETS: &[&[u8]; 3] = &[
    include_bytes!("../assets/traffic-low.png"),
    include_bytes!("../assets/traffic-low.png"),
    include_bytes!("../assets/traffic-high.png"),
];

// Hazard
const ALERT_TYPE_HAZARD: &[&str; 8] = &[
    DEFAULT,
    "HAZARD_ON_ROAD_POT_HOLE",
    "HAZARD_ON_ROAD_CONSTRUCTION",
    "HAZARD_ON_ROAD_ICE",
    "HAZARD_ON_ROAD_TRAFFIC_LIGHT_FAULT",
    "HAZARD_ON_ROAD_OBJECT",
    "HAZARD_ON_SHOULDER_CAR_STOPPED",
    "HAZARD_WEATHER_FOG",
];
const ALERT_TYPE_HAZARD_ASSETS: &[&[u8]; 8] = &[
    include_bytes!("../assets/hazard.png"),
    include_bytes!("../assets/pothole.png"),
    include_bytes!("../assets/construction.png"),
    include_bytes!("../assets/ice.png"),
    include_bytes!("../assets/light.png"),
    include_bytes!("../assets/object.png"),
    include_bytes!("../assets/vehicle-stopped.png"),
    include_bytes!("../assets/fog.png"),
];

// Closed
const ALERT_ROAD_CLOSED: &[&str; 1] = &[DEFAULT];
const ALERT_ROAD_CLOSED_ASSETS: &[&[u8]; 1] = &[include_bytes!("../assets/closure.png")];

// Accident
const ALERT_ACCIDENT: &[&str; 1] = &[DEFAULT];
const ALERT_ACCIDENT_ASSETS: &[&[u8]; 1] = &[include_bytes!("../assets/accident.png")];

// Police
const ALERT_POLICE: &[&str; 1] = &[DEFAULT];
const ALERT_POLICE_ASSETS: &[&[u8]; 1] = &[include_bytes!("../assets/police.png")];

// Default
const ALERT_DEFAULT: &[&str; 1] = &[DEFAULT];
const ALERT_DEFAULT_ASSETS: &[&[u8]; 1] = &[include_bytes!("../assets/simple.png")];

// Correlation
const ALERTS: [&str; 6] = [
    "JAM",
    "HAZARD",
    "ROAD_CLOSED",
    "ACCIDENT",
    "POLICE",
    DEFAULT,
];
const SUB_ALERTS: [&[&str]; 6] = [
    ALERT_TYPE_JAM,
    ALERT_TYPE_HAZARD,
    ALERT_ROAD_CLOSED,
    ALERT_ACCIDENT,
    ALERT_POLICE,
    ALERT_DEFAULT,
];
const SUB_ALERTS_ASSETS: [&[&[u8]]; 6] = [
    ALERT_TYPE_JAM_ASSETS,
    ALERT_TYPE_HAZARD_ASSETS,
    ALERT_ROAD_CLOSED_ASSETS,
    ALERT_ACCIDENT_ASSETS,
    ALERT_POLICE_ASSETS,
    ALERT_DEFAULT_ASSETS,
];

/**********
 * Finder *
 **********/

pub fn find_alert_asset<'start, 'end>(
    main_type: &'start str,
    sub_type: &'start str,
) -> Option<&'end [u8]> {
    // Get Type
    let found_main_type = ALERTS.iter().position(|&each| each == main_type);

    // Return if not found the first one
    if found_main_type.is_none() {
        return None;
    }

    // Otherwise we have a chance to narrow down
    let found_sub_type = SUB_ALERTS[found_main_type?]
        .iter()
        .position(|&each| each == sub_type);

    // Check which we found
    if found_sub_type.is_none() {
        return Some(SUB_ALERTS_ASSETS[found_main_type?][0]);
    }

    Some(SUB_ALERTS_ASSETS[found_main_type?][found_sub_type?])
}
