/***********
 * Imports *
 ***********/

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
// Standard
use std::net::{Ipv4Addr, SocketAddr};

// Utilities
mod cross;
mod getter;
mod paint;
mod print;
mod utils;

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

    // Extract the alerts from the list
    let alerts = getter::alerts_extract(&data.await);

    // Alerts to its own tile
    let tiled = paint::alerts_to_tile(&alerts, &spacer);

    // Join all tiles
    let joined = paint::join_tiles(&tiles.await, &tiled);

    // Extract bytes from picture
    let bytes = paint::png_bytes(&joined);

    // Response
    Response::builder()
        .status(StatusCode::OK)
        .header(header::SERVER, NAME)
        .header(header::CONTENT_TYPE, "image/png")
        .body(Body::from(bytes))
        .unwrap()
}
