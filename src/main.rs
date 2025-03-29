/***********
 * Imports *
 ***********/

// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_mut)]
// #![allow(dead_code)]

// HTTP
use axum::{
    body::Body,
    extract::{ConnectInfo, Query},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
// Data
use serde::Deserialize;
// Image
use image::{
    imageops, ColorType, DynamicImage, GenericImageView, ImageBuffer, ImageFormat, RgbaImage,
};
// Standard
use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::net::{Ipv4Addr, SocketAddr};

// Utilities
mod cross;
mod getter;
mod utils;

/*************
 * Constants *
 *************/

// Address
const ADDRESS: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 8080;
const NAME: &str = "mapper";

const STRFTIME: &str = "%y-%m-%d %T";

/**************
 * Structures *
 **************/

struct Alert<'together> {
    icon: &'together [u8],
    lat: f64,
    lon: f64,
}

/*********
 * Cache *
 *********/

// static mut cloud: HashMap<&str, Vec<Alert>>;

/********
 * Main *
 ********/

#[tokio::main]
async fn main() {
    // Build Application
    let app = Router::new()
        // Default Route
        .route("/", get(default));

    // Create listener
    let bind: String = format!("{}:{}", ADDRESS, PORT);
    let listener = tokio::net::TcpListener::bind(bind.as_str()).await.unwrap();

    // Verbose
    println!("Listening on {}", bind.as_str());

    // Server It
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/***********
 * Handler *
 ***********/

// Query
#[derive(Deserialize)]
struct Coordinates {
    x: u32,
    y: u32,
    z: u16,
}

// Basic
async fn default(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(raw_agent): TypedHeader<UserAgent>,
    arguments: Query<Coordinates>,
) -> Response {
    // User Agent
    let user_agent = raw_agent.to_string();

    // Calculate Coordinates
    let (top, left) = utils::xyz_to_coordinate(
        ((arguments.x as i32) + getter::WAZ_PAD_X.iter().min().unwrap_or(&0)) as u32,
        ((arguments.y as i32) + getter::WAZ_PAD_Y.iter().min().unwrap_or(&0)) as u32,
        arguments.z,
    );
    let (bottom, right) = utils::xyz_to_coordinate(
        ((arguments.x as i32) + getter::WAZ_PAD_X.iter().max().unwrap_or(&0)) as u32 + 1,
        ((arguments.y as i32) + getter::WAZ_PAD_Y.iter().max().unwrap_or(&0)) as u32 + 1,
        arguments.z,
    );

    // Fetch GeoJSON
    let data = getter::get_jsons(&user_agent, top, left, bottom, right);

    // Start to get all tiles
    let tiles = getter::get_tiles(&user_agent, arguments.x, arguments.y, arguments.z);

    // Current time
    let now = chrono::Utc::now();
    // Verbose
    println!(
        "[{}] - <<< - {} - {}",
        now.format(STRFTIME).to_string(),
        addr,
        user_agent
    );

    /* TODO
     * Wait for all the layers to download and splice them together
     * Use the empty one first
     */

    // Create our blank canvas
    // Sizes
    let image_width = cross::TILE_SIZE * getter::WAZ_PAD_X.len();
    let image_height = cross::TILE_SIZE * getter::WAZ_PAD_Y.len();
    // Create
    let mut canvas: RgbaImage = ImageBuffer::new(image_width as u32, image_height as u32);

    // Create local list of alerts
    let mut tidy_alerts: Vec<Alert> = Vec::new();

    // Wait
    let json = data.await;

    // Iterate alerts
    if let Some(alerts) = json["alerts"].as_array() {
        for alert in alerts.iter() {
            // Get the icon reference
            let icon_reference = cross::find_alert_asset(
                alert["type"].as_str().unwrap(),
                alert["subtype"].as_str().unwrap(),
            );
            // Create alert
            let item_alert = Alert {
                icon: icon_reference,
                lat: alert["location"]["y"].as_f64().unwrap(),
                lon: alert["location"]["x"].as_f64().unwrap(),
            };

            // Add to the vector
            tidy_alerts.push(item_alert);
        }
    }

    // Sort vector
    tidy_alerts.sort_by(|after, before| {
        if before.lat == after.lat {
            before.lon.partial_cmp(&after.lon).unwrap()
        } else {
            before.lat.partial_cmp(&after.lat).unwrap()
        }
    });

    // Add the alerts to the canvas
    for alert in tidy_alerts.iter() {
        // Translate the coordinates
        let (confined_x, confined_y) = utils::coordinates_confine(
            alert.lat,
            alert.lon,
            top,
            left,
            bottom,
            right,
            image_width as u32,
            image_height as u32,
        );

        // Load icon
        let current_icon = image::load_from_memory(alert.icon).unwrap();
        let (width, height) = current_icon.dimensions();

        // Fix edges
        let (edge_x, edge_y) = utils::translate_edge(
            width,
            height,
            confined_x,
            confined_y,
            cross::ICON_RATIO_X,
            cross::ICON_RATIO_Y,
        );

        // Overlay it
        imageops::overlay(&mut canvas, &current_icon, edge_x as i64, edge_y as i64);
    }

    // Crop to selection
    let crop_x = getter::WAZ_PAD_X
        .iter()
        .position(|&each| each == 0)
        .unwrap_or(0)
        * cross::TILE_SIZE;
    let crop_y = getter::WAZ_PAD_Y
        .iter()
        .position(|&each| each == 0)
        .unwrap_or(0)
        * cross::TILE_SIZE;
    let crop = imageops::crop(
        &mut canvas,
        crop_x as u32,
        crop_y as u32,
        cross::TILE_SIZE as u32,
        cross::TILE_SIZE as u32,
    )
    .to_image();

    // Create final images
    let mut final_image = DynamicImage::new(
        cross::TILE_SIZE as u32,
        cross::TILE_SIZE as u32,
        ColorType::Rgba8,
    );

    // Add each tile
    for tile in tiles.await.iter() {
        imageops::overlay(
            &mut final_image,
            &image::load_from_memory(&tile).unwrap(),
            0,
            0,
        );
    }

    // Add copped image
    imageops::overlay(&mut final_image, &crop, 0, 0);

    // Buffer
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    final_image.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let bytes: Vec<u8> = buffer.into_inner().unwrap().into_inner();

    // Response
    Response::builder()
        .status(StatusCode::OK)
        .header(header::SERVER, NAME)
        .header(header::CONTENT_TYPE, "image/png")
        .body(Body::from(bytes))
        .unwrap()
}
