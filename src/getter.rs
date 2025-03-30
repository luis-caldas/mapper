/***********
 * Imports *
 ***********/

// Mine
use crate::print;
use crate::utils;

/*************
 * Variables *
 *************/

// XYZ Maps
const LINKS: [&str; 1] = ["https://mts0.google.com/vt/lyrs=h,traffic&x={x}&y={y}&z={z}&style=3"];

// WAZ
const WAZ: &str = "https://embed.waze.com/live-map/api/georss?env=row&types=alerts&top={top}&bottom={bottom}&left={left}&right={right}";

// Locators
const IN_ALERTS: &str = "alerts";
const IN_TYPE: &str = "type";
const IN_SUBTYPE: &str = "subtype";
const IN_LOCATION: &str = "location";
const IN_LOCATION_X: &str = "x";
const IN_LOCATION_Y: &str = "y";

/***********
 * Structs *
 ***********/

#[derive(Clone)]
pub struct Alert {
    pub icon: String,
    pub subicon: String,
    pub position: utils::Coordinate,
}

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
    print::print_out_xyz(&position);

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
    print::print_out_plot(&position);

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

pub fn alerts_extract(json: &serde_json::Value) -> Vec<Alert> {
    // Create local list of alerts
    let mut tidy: Vec<Alert> = Vec::new();

    // Iterate alerts
    if let Some(alerts) = json[IN_ALERTS].as_array() {
        for alert in alerts.iter() {
            // Create alert
            let item_alert = Alert {
                icon: alert[IN_TYPE].as_str().unwrap().to_string(),
                subicon: alert[IN_SUBTYPE].as_str().unwrap().to_string(),
                position: utils::Coordinate {
                    lat: alert[IN_LOCATION][IN_LOCATION_Y].as_f64().unwrap(),
                    lon: alert[IN_LOCATION][IN_LOCATION_X].as_f64().unwrap(),
                },
            };

            // Add to the vector
            tidy.push(item_alert);
        }
    }

    // Sort vector
    tidy.sort_by(|after, before| {
        if before.position.lat == after.position.lat {
            before
                .position
                .lon
                .partial_cmp(&after.position.lon)
                .unwrap()
        } else {
            before
                .position
                .lat
                .partial_cmp(&after.position.lat)
                .unwrap()
        }
    });

    tidy
}
