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
