/*************
 * Variables *
 *************/

// XYZ Maps
const LINKS: [&str; 1] = ["https://mts0.google.com/vt/lyrs=h,traffic&x={x}&y={y}&z={z}&style=3"];

// WAZ
const WAZ: &str = "https://embed.waze.com/live-map/api/georss?env=row&types=alerts&top={top}&bottom={bottom}&left={left}&right={right}";

// Offsets around tile
pub const WAZ_PAD_X: [i32; 3] = [-1, 0, 1];
pub const WAZ_PAD_Y: [i32; 3] = [-1, 0, 1];

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

fn replace_url_waz(input: &str, top: f64, left: f64, bottom: f64, right: f64) -> String {
    input
        .replace("{top}", &top.to_string())
        .replace("{left}", &left.to_string())
        .replace("{bottom}", &bottom.to_string())
        .replace("{right}", &right.to_string())
}

fn replace_url(input: &str, x: u32, y: u32, z: u16) -> String {
    input
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string())
        .replace("{z}", &z.to_string())
}

pub async fn get_tiles(user_agent: &str, x: u32, y: u32, z: u16) -> Vec<Vec<u8>> {
    // URLs
    let mut urls: Vec<String> = Vec::new();
    for url in LINKS.iter() {
        urls.push(replace_url(&url, x, y, z));
    }

    // Promises
    let mut promises = Vec::new();
    for url in urls.iter() {
        promises.push(get_tile(&url, &user_agent));
    }

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

    // Verbose
    let now = chrono::Utc::now();
    println!(
        "[{}] - >>> - {} - {}",
        now.format(crate::STRFTIME).to_string(),
        url,
        user_agent
    );

    Ok(bytes.to_vec())
}

pub async fn get_jsons(
    user_agent: &str,
    top: f64,
    left: f64,
    bottom: f64,
    right: f64,
) -> serde_json::Value {
    // URLs
    let url = replace_url_waz(&WAZ, top, left, bottom, right);

    // Promise
    let promise = get_json(&url, &user_agent);

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

    // Verbose
    let now = chrono::Utc::now();
    println!(
        "[{}] - >>> - {} - {}",
        now.format(crate::STRFTIME).to_string(),
        url,
        user_agent
    );

    Ok(json)
}
