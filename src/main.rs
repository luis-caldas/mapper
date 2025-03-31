/***********
 * Imports *
 ***********/

// Async
use tokio::{task, time};
// HTTP
use axum::{
    body::Body,
    extract::{ConnectInfo, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
// Data
use serde::Deserialize;
// Cache
use moka::future::Cache;
// Time
use std::time::Duration;
// Standard
use std::net::{Ipv4Addr, SocketAddr};

// Utilities
mod cache;
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

// Main
#[tokio::main]
async fn main() {
    // Cache
    let cloud: Cache<utils::XYZ, Vec<getter::Alert>> = Cache::new(cache::CACHE_MAX);
    let clean_cloud = cloud.clone();
    let tiloud: Cache<utils::XYZ, Vec<Vec<Vec<u8>>>> = Cache::new(cache::CACHE_MAX);
    let clean_tiloud = cloud.clone();

    // Clear cache periodically
    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(cache::CACHE_TTL));
        loop {
            // Wait for the given interval
            interval.tick().await;
            // Clear cache
            clean_cloud.invalidate_all();
            clean_tiloud.invalidate_all();
        }
    });

    // Build Web Application
    let app = Router::new()
        // Default Route
        .route("/", get(default))
        .with_state((cloud.clone(), tiloud.clone()));

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

    forever.await.unwrap();
}

/***********
 * Handler *
 ***********/

// Basic
async fn default(
    State((cached, tiled)): State<(Cache<utils::XYZ, Vec<getter::Alert>>, Cache<utils::XYZ, Vec<Vec<Vec<u8>>>>)>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(raw_agent): TypedHeader<UserAgent>,
    arguments: Query<Arguments>,
) -> Response {
    // User Agent
    let user_agent = raw_agent.to_string();

    // Convert inputs
    let given_xyz = utils::XYZ {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };

    // Start tiles straight away
    let data_tiles = getter::get_tiles(&user_agent, &given_xyz);

    // Zoom out for a larger cached area and grow it
    let cache_area = utils::zoom_scale(cache::CACHE_ZOOM, &given_xyz);
    let cache_spaced = utils::grow_pad(utils::TILE_OFFSET, &cache_area);
    // Generic big area that we will actually use for painting
    let pings_spaced = utils::grow_pad(utils::TILE_OFFSET, &given_xyz);

    // Look for cache
    let cache_alerts = cached.get(&cache_area).await;
    let pings_chosen = async {
        match cache_alerts {
            Some(something) => something,
            None => {
                let data = getter::get_jsons(&user_agent, &cache_spaced);
                let extracted = getter::alerts_extract(&data.await);
                cached.insert(cache_area, extracted.clone()).await;
                extracted
            }
        }
    };

    // Verbose
    print::print_in(&addr.to_string(), &user_agent);

    // Extract only the needed area
    let pings_area = cache::find_alerts(&pings_chosen.await, &pings_spaced);

    // Alerts to its own tile
    let tiles_alerts = paint::alerts_to_tile(&pings_area, &pings_spaced);

    // Join all tiles & extract its bytes
    let tiles_joined = paint::join_tiles(&data_tiles.await, &tiles_alerts);
    let tiles_bytes = paint::png_bytes(&tiles_joined);

    // Response
    Response::builder()
        .status(StatusCode::OK)
        .header(header::SERVER, NAME)
        .header(header::CONTENT_TYPE, "image/png")
        .body(Body::from(tiles_bytes))
        .unwrap()
}
