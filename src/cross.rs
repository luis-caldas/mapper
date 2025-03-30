/************
 * Includes *
 ************/

use crate::utils;

/*************
 * Variables *
 *************/

// Icons
pub const ICON_POINT: utils::Ratios = utils::Ratios {
    x: 0.5,
    y: 1.0,
};

/**********
 * Macros *
 **********/

// Get asset
macro_rules! bytes_asset {
    ($name:literal) => {
        include_bytes!(concat!("icons/", $name, ".png"))
    };
}

/**************
 * Assignment *
 **************/

// Default
const DEFAULT: &str = "DEFAULT";
const DEFAULT_INDEX: usize = 0;

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
    bytes_asset!("hazard"),
    bytes_asset!("pothole"),
    bytes_asset!("construction"),
    bytes_asset!("ice"),
    bytes_asset!("light"),
    bytes_asset!("object"),
    bytes_asset!("vehicle-stopped"),
    bytes_asset!("fog"),
];

// Jam
const ALERT_TYPE_JAM: &[&str; 3] = &[DEFAULT, "JAM_HEAVY_TRAFFIC", "JAM_STAND_STILL_TRAFFIC"];
const ALERT_TYPE_JAM_ASSETS: &[&[u8]; 3] = &[
    bytes_asset!("traffic-low"),
    bytes_asset!("traffic-low"),
    bytes_asset!("traffic-high"),
];

// Closed
const ALERT_ROAD_CLOSED: &[&str; 1] = &[DEFAULT];
const ALERT_ROAD_CLOSED_ASSETS: &[&[u8]; 1] = &[bytes_asset!("closure")];

// Accident
const ALERT_ACCIDENT: &[&str; 1] = &[DEFAULT];
const ALERT_ACCIDENT_ASSETS: &[&[u8]; 1] = &[bytes_asset!("accident")];

// Police
const ALERT_POLICE: &[&str; 1] = &[DEFAULT];
const ALERT_POLICE_ASSETS: &[&[u8]; 1] = &[bytes_asset!("police")];

// Correlation
const ALERTS: [&str; 5] = ["HAZARD", "JAM", "ROAD_CLOSED", "ACCIDENT", "POLICE"];
const SUB_ALERTS: [&[&str]; 5] = [
    ALERT_TYPE_JAM,
    ALERT_TYPE_HAZARD,
    ALERT_ROAD_CLOSED,
    ALERT_ACCIDENT,
    ALERT_POLICE,
];
const SUB_ALERTS_ASSETS: [&[&[u8]]; 5] = [
    ALERT_TYPE_JAM_ASSETS,
    ALERT_TYPE_HAZARD_ASSETS,
    ALERT_ROAD_CLOSED_ASSETS,
    ALERT_ACCIDENT_ASSETS,
    ALERT_POLICE_ASSETS,
];

/**********
 * Finder *
 **********/

pub fn find_alert_asset<'start, 'end>(main_type: &'start str, sub_type: &'start str) -> &'end [u8] {
    // Get Type
    let found_main_type = ALERTS
        .iter()
        .position(|&each| each == main_type)
        .unwrap_or(DEFAULT_INDEX);

    // Otherwise we have a chance to narrow down
    let found_sub_type = SUB_ALERTS[found_main_type]
        .iter()
        .position(|&each| each == sub_type)
        .unwrap_or(DEFAULT_INDEX);

    SUB_ALERTS_ASSETS[found_main_type][found_sub_type]
}
