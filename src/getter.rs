/***********
 * Imports *
 ***********/

use crate::utils;

/*************
 * Variables *
 *************/

// Verbose
pub const PRINT_COMING: &str = "<<<";
pub const PRINT_GOING: &str = ">>>";

// XYZ Maps
const LINKS: [&str; 1] = ["https://mts0.google.com/vt/lyrs=h,traffic&x={x}&y={y}&z={z}&style=3"];

// WAZ
const WAZ: &str = "https://embed.waze.com/live-map/api/georss?env=row&types=alerts&top={top}&bottom={bottom}&left={left}&right={right}";

// Offsets around tile
pub const WAZ_OFFSET: u32 = 1;
pub const WAZ_OFFSET_LENGTH: u32 = (WAZ_OFFSET * 2) + 1;

// Cache
pub const CACHE_ZOOM: u16 = 10; // XYZ - Z
pub const CACHE_TTL: u16 = 60; // Seconds

// Locators
pub const IN_ALERTS: &str = "alerts";
pub const IN_TYPE: &str = "type";
pub const IN_SUBTYPE: &str = "subtype";
pub const IN_LOCATION: &str = "location";
pub const IN_LOCATION_X: &str = "x";
pub const IN_LOCATION_Y: &str = "y";

/*************
 * Functions *
 *************/

fn replace_url_waz(input: &str, position: &utils::Plot) -> String {
    input
        .replace("{top}", &position.top.lat.to_string())
        .replace("{left}", &position.top.lon.to_string())
        .replace("{bottom}", &position.bottom.lat.to_string())
        .replace("{right}", &position.bottom.lon.to_string())
}

fn replace_url(input: &str, position: &utils::XYZ) -> String {
    input
        .replace("{x}", &position.x.to_string())
        .replace("{y}", &position.y.to_string())
        .replace("{z}", &position.z.to_string())
}

pub async fn get_tiles(user_agent: &str, position: &utils::XYZ) -> Vec<Vec<u8>> {
    // URLs
    let mut urls: Vec<String> = Vec::new();
    for url in LINKS.iter() {
        urls.push(replace_url(&url, &position));
    }

    // Promises
    let mut promises = Vec::new();
    for url in urls.iter() {
        promises.push(get_tile(&url, &user_agent));
    }

    // Verbose
    let now = chrono::Utc::now();
    println!(
        "[{}] - {} - Tile - {}, {}, {}",
        now.format(crate::STRFTIME).to_string(),
        colored::Colorize::red(PRINT_GOING),
        position.x,
        position.y,
        position.z,
    );

    // Tiles
    let mut tiles = Vec::new();
    for promise in promises {
        let tile = promise.await;
        if tile.is_ok() {
            tiles.push(tile.unwrap());
        }
    }

    tiles
}

pub async fn get_tile(url: &str, user_agent: &str) -> Result<Vec<u8>, reqwest::Error> {
    // Client
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;

    // Response
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    Ok(bytes.to_vec())
}

pub async fn get_jsons(user_agent: &str, position: &utils::Plot) -> serde_json::Value {
    // URLs
    let url = replace_url_waz(&WAZ, &position);

    // Promise
    let promise = get_json(&url, &user_agent);

    // Verbose
    let now = chrono::Utc::now();
    println!(
        "[{}] - {} - Tile - {}, {}, {}, {}",
        now.format(crate::STRFTIME).to_string(),
        colored::Colorize::red(PRINT_GOING),
        position.top.lat,
        position.top.lon,
        position.bottom.lat,
        position.bottom.lon,
    );

    // Data
    promise.await.unwrap_or(serde_json::json!({}))
}

pub async fn get_json(url: &str, user_agent: &str) -> Result<serde_json::Value, reqwest::Error> {
    // Client
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .gzip(true)
        .build()?;

    // Response
    let response = client.get(url).send().await?;
    let json = response.json::<serde_json::Value>().await?;

    Ok(json)
}
