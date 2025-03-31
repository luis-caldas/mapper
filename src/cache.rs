/***********
 * Imports *
 ***********/

// Mine
use crate::getter;
use crate::utils;

/*************
 * Variables *
 *************/

// Cache
pub const CACHE_MAX: u64 = 0xFFFF;
pub const CACHE_ZOOM: u16 = 10; // XYZ - Z
pub const CACHE_TTL: u64 = 60; // Seconds

/*************
 * Functions *
 *************/

// Find all alerts in an area
pub fn find_alerts(map: &Vec<getter::Alert>, block: &utils::Plot) -> Vec<getter::Alert> {
    // Initialise new vector
    let mut found: Vec<getter::Alert> = Vec::new();

    // Iterate the given map
    for alert in map.iter() {
        // Check bounds
        if alert.position.lat < block.top.lat
            && alert.position.lat > block.bottom.lat
            && alert.position.lon < block.bottom.lon
            && alert.position.lon > block.top.lon
        {
            found.push(alert.clone());
        }
    }

    found
}
