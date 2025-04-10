/***********
 * Imports *
 ***********/

// Mine
use crate::cross;
use crate::getter;
use crate::utils;

// Image
use image::{imageops, ImageFormat, RgbaImage};

// Bytes
use std::io::{BufWriter, Cursor};

/*************
 * Variables *
 *************/

/***********
 * Structs *
 ***********/

/*************
 * Functions *
 *************/

pub fn alerts_to_tile(alerts: &Vec<getter::Alert>, spacer: &utils::Plot) -> RgbaImage {
    // Create our blank canvas
    let mut canvas = RgbaImage::new(utils::TILE_INFLATED, utils::TILE_INFLATED);

    // Size structure
    let canvas_size: utils::Raster = utils::Raster {
        x: canvas.width(),
        y: canvas.height(),
    };

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
        let edges = utils::translate_edge(&icon_dimensions, &confined);

        // Overlay it
        imageops::overlay(&mut canvas, &icon_current, edges.x as i64, edges.y as i64);
    }

    // Cropped to desired size
    RgbaImage::from(
        imageops::crop(
            &mut canvas,
            utils::TILE_ORIGINAL_START,
            utils::TILE_ORIGINAL_START,
            utils::TILE_SIZE,
            utils::TILE_SIZE,
        )
        .to_image(),
    )
}

pub async fn join_quadrant_tiles(tiles: &Vec<Vec<Vec<u8>>>) -> Vec<Vec<u8>> {
    // Base
    let mut all_tiles: Vec<Vec<u8>> = Vec::new();

    // Add each tile
    for tile in tiles.iter() {
        let image = if tile.len() == 1 {
            image::load_from_memory(&tile.first().unwrap())
                .unwrap()
                .to_rgba8()
        } else {
            // New joined
            let mut joiner = RgbaImage::new(utils::TILE_SIZE, utils::TILE_SIZE);
            // Load images
            let top_left = image::load_from_memory(&tile[0]).unwrap();
            let top_right = image::load_from_memory(&tile[1]).unwrap();
            let bottom_left = image::load_from_memory(&tile[2]).unwrap();
            let bottom_right = image::load_from_memory(&tile[3]).unwrap();
            // Size
            let size = top_left.width().into();
            // Overlay each
            imageops::overlay(&mut joiner, &top_left, 0, 0);
            imageops::overlay(&mut joiner, &top_right, size, 0);
            imageops::overlay(&mut joiner, &bottom_left, 0, size);
            imageops::overlay(&mut joiner, &bottom_right, size, size);

            joiner
        };

        // Stretch the tile if needed
        let insert_image = if image.width() != utils::TILE_SIZE {
            imageops::resize(
                &image,
                utils::TILE_SIZE,
                utils::TILE_SIZE,
                imageops::FilterType::Nearest,
            )
        } else {
            image
        };

        let png_data = png_bytes(&insert_image);
        all_tiles.push(png_data);
    }

    all_tiles
}

pub fn join_tiles(tiles: &Vec<Vec<u8>>, tiled: &RgbaImage) -> RgbaImage {
    // Base
    let mut base = RgbaImage::new(utils::TILE_SIZE, utils::TILE_SIZE);

    // Add each tile
    for tile in tiles.iter() {
        let image = image::load_from_memory(&tile).unwrap();
        if image.width() != utils::TILE_SIZE {
            let up = imageops::resize(
                &image,
                utils::TILE_SIZE,
                utils::TILE_SIZE,
                imageops::FilterType::Nearest,
            );
            imageops::overlay(&mut base, &up, 0, 0);
        } else {
            imageops::overlay(&mut base, &image, 0, 0);
        }
    }

    // Add the last overlay tile
    imageops::overlay(&mut base, tiled, 0, 0);

    base
}

pub fn png_bytes(image: &RgbaImage) -> Vec<u8> {
    // Buffer
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));

    // Write image to buffer with format
    image.write_to(&mut buffer, ImageFormat::Png).unwrap();

    // Get bytes
    buffer.into_inner().unwrap().into_inner()
}
