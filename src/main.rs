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
use std::io::{BufWriter, Cursor};
use std::net::{Ipv4Addr, SocketAddr};
// use std::collections::HashMap;
// use std::sync::Mutex;

// Utilities
mod cross;
mod getter;
mod utils;
mod print;

/*************
 * Constants *
 *************/

// Address
const ADDRESS: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PORT: u16 = 8080;
const NAME: &str = "mapper";

/**************
 * Structures *
 **************/

// Query
#[derive(Deserialize)]
struct Arguments {
    x: u32,
    y: u32,
    z: u16,
}

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

// Basic
async fn default(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(raw_agent): TypedHeader<UserAgent>,
    arguments: Query<Arguments>,
) -> Response {
    // User Agent
    let user_agent = raw_agent.to_string();

    // Convert inputs
    let given = utils::XYZ {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };

    // Zoom out for a larger cached area
    // let cache_area = zoom_scale(getter::CACHE_ZOOM, &given);

    // Grow search area so we have left overs around
    let spacer = utils::grow_pad(utils::TILE_OFFSET, &given);

    // Start async fetchers
    let data = getter::get_jsons(&user_agent, &spacer);
    let tiles = getter::get_tiles(&user_agent, &given);

    // Verbose
    print::print_in(&addr.to_string(), &user_agent);

    // Create our blank canvas
    let canvas_size = utils::Raster {
        x: utils::TILE_INFLATED,
        y: utils::TILE_INFLATED,
    };
    let mut canvas: RgbaImage = ImageBuffer::new(canvas_size.x, canvas_size.y);

    // Extract the alerts from the list
    let alerts = getter::alerts_extract(&data.await);

    // Add the alerts to the canvas
    for alert in alerts.iter() {
        // Translate the coordinates
        let confined = utils::coordinates_confine(&alert.position, &spacer, &canvas_size);

        // Load icon
        let icon_bytes = cross::find_alert_asset(&alert.icon, &alert.subicon);
        let icon_current = image::load_from_memory(icon_bytes).unwrap();
        let icon_dimensions = utils::Raster {
            x: icon_current.width(),
            y: icon_current.height(),
        };

        // Fix edges
        let edges = utils::translate_edge(
            &icon_dimensions,
            &confined,
        );

        // Overlay it
        imageops::overlay(&mut canvas, &icon_current, edges.x as i64, edges.y as i64);
    }

    // Crop to real tile
    let crop = imageops::crop(
        &mut canvas,
        utils::TILE_ORIGINAL_START,
        utils::TILE_ORIGINAL_START,
        utils::TILE_SIZE,
        utils::TILE_SIZE,
    )
    .to_image();

    // Create final images
    let mut final_image = DynamicImage::new(utils::TILE_SIZE, utils::TILE_SIZE, ColorType::Rgba8);

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
