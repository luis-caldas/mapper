/*************
 * Variables *
 *************/

// XYZ Maps
const DEFAULT: usize = 0;
pub const LIST: [&str; 1] = ["Google"];
pub const LINKS: [&str; 1] =
    ["https://mts0.google.com/vt/lyrs=h,traffic&x={x}&y={y}&z={z}&style=3"];

// WAZ
pub const WAZ: &str = "https://embed.waze.com/live-map/api/georss?env=row&types=alerts&top={top}&bottom={bottom}&left={left}&right={right}";

// Offsets around tile
pub const WAZ_PAD_X: [i32; 3] = [-1, 0, 1];
pub const WAZ_PAD_Y: [i32; 3] = [-1, 0, 1];

// Cache
pub const CACHE_ZOOM: u16 = 10;  // XYZ - Z
pub const CACHE_TTL: u16 = 60;  // Seconds

/*************
 * Functions *
 *************/

pub fn replace_url_waz(input: &str, top: f64, left: f64, bottom: f64, right: f64) -> String {
    input
        .to_owned()
        .replace("{top}", &top.to_string())
        .replace("{left}", &left.to_string())
        .replace("{bottom}", &bottom.to_string())
        .replace("{right}", &right.to_string())
}

pub fn replace_url(input: &str, x: u32, y: u32, z: u16) -> String {
    input
        .to_owned()
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string())
        .replace("{z}", &z.to_string())
}

pub fn find_url(name: &str) -> &str {
    // Find it
    let found = LIST.iter().position(|&each| each == name).unwrap_or(DEFAULT);

    LINKS[found]
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

pub async fn get_geojson(url: &str, user_agent: &str) -> Result<serde_json::Value, reqwest::Error> {
    // Client
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;

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
