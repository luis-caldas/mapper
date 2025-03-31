/***********
 * Imports *
 ***********/

// Time
use std::time::Duration;
use tokio::time;

// Cache
use moka::future::Cache;

// Traits
use std::hash::Hash;

// Mine
use crate::getter;
use crate::utils;

/*************
 * Variables *
 *************/

// Cache
pub const CACHE_MAX: u64 = 0xFFFF;
pub const CACHE_ZOOM: u16 = 10; // XYZ - Z
pub const CACHE_TTL_TILE: u64 = 60; // Seconds
pub const CACHE_TTL_DATA: u64 = 360; // Seconds

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

pub async fn clean_cache<K: Hash + Eq + Send + Sync + 'static, V: Clone + Send + Sync + 'static>(
    cache: Cache<K, V>,
    seconds: u64,
) -> () {
    let mut interval = time::interval(Duration::from_secs(seconds));
    loop {
        // Wait for the given interval
        interval.tick().await;
        // Clear cache
        cache.invalidate_all();
    }
}
