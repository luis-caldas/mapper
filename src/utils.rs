/***********
 * Imports *
 ***********/

use std::cmp;
use std::f64::consts;

/**************
 * Structures *
 **************/

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct XYZ {
    pub x: u32,
    pub y: u32,
    pub z: u16,
}

#[derive(Clone)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

pub struct Plot {
    pub top: Coordinate,
    pub bottom: Coordinate,
}

pub struct Raster {
    pub x: u32,
    pub y: u32,
}

pub struct Ratios {
    pub x: f64,
    pub y: f64,
}

/*************
 * Variables *
 *************/

// Tiling
pub const TILE_SIZE: u32 = 512; // Pixels

// Offsets around tile
pub const TILE_OFFSET: u32 = 1;
pub const TILE_OFFSET_LENGTH: u32 = (TILE_OFFSET * 2) + 1;

// Tile full inflated length
pub const TILE_INFLATED: u32 = TILE_OFFSET_LENGTH * TILE_SIZE;

// Start of original tile in an offset situation
pub const TILE_ORIGINAL_START: u32 = TILE_OFFSET * TILE_SIZE;

// Location of the point on an icon
const ICON_POINT: Ratios = Ratios { x: 0.5, y: 1.0 };

/*************
 * Functions *
 *************/

// Change zoom scale
pub fn zoom_scale(new_z: u16, pane: &XYZ) -> XYZ {
    // Check if we need to scale
    if pane.z <= new_z {
        return pane.clone();
    }

    // Get Correlation
    let correlation = i32::from(i32::from(pane.z) - i32::from(new_z));

    // New Coordinates
    let new_x = f64::from(pane.x) / 2_f64.powi(correlation);
    let new_y = f64::from(pane.y) / 2_f64.powi(correlation);

    XYZ {
        x: new_x.floor() as u32,
        y: new_y.floor() as u32,
        z: new_z,
    }
}

// XYZ -> Lat & Lon
pub fn xyz_to_coordinate(pane: &XYZ) -> Coordinate {
    // Size
    let n = 2_f64.powi(i32::from(pane.z));

    // Longitude
    let longitude = ((f64::from(pane.x) / n) * 360.0) - 180.0;

    // Latitude
    let latitude = (180.0 / consts::PI)
        * (consts::PI * (1.0 - 2.0 * (f64::from(pane.y) / n)))
            .sinh()
            .atan();

    // Return
    Coordinate {
        lat: latitude,
        lon: longitude,
    }
}

// Translate coordinates
pub fn coordinates_confine(item: &Coordinate, confine: &Plot, dest: &Raster) -> Raster {
    // Get ratios
    let ratio_x: f64 = f64::from(dest.x) / (confine.top.lon - confine.bottom.lon);
    let ratio_y: f64 = f64::from(dest.y) / (confine.top.lat - confine.bottom.lat);

    // Offset items
    let item_x: f64 = (confine.top.lon - item.lon) * ratio_x;
    let item_y: f64 = (confine.top.lat - item.lat) * ratio_y;

    Raster {
        x: item_x.floor() as u32,
        y: item_y.floor() as u32,
    }
}

// Fix for images to represent a centre
pub fn translate_edge(dimensions: &Raster, position: &Raster) -> Raster {
    // Offsets
    let offset_x: u32 = (f64::from(dimensions.x) * ICON_POINT.x) as u32;
    let offset_y: u32 = (f64::from(dimensions.y) * ICON_POINT.y) as u32;

    // Translate
    let translated_x: u32 = if position.x > offset_x {
        position.x - offset_x
    } else {
        0
    };
    let translated_y: u32 = if position.y > offset_y {
        position.y - offset_y
    } else {
        0
    };

    Raster {
        x: translated_x,
        y: translated_y,
    }
}

// Grow a pad by factors
pub fn grow_pad(offset: u32, pane: &XYZ) -> Plot {
    let min_tile = 0_i32;
    let max_tile = 2_u32.pow(u32::from(pane.z));

    // Check boundaries
    let top_x = cmp::max(min_tile, (pane.x as i32) - (offset as i32));
    let top_y = cmp::max(min_tile, (pane.y as i32) - (offset as i32));
    let bottom_x = cmp::min(max_tile, (pane.x + offset) as u32 + 1);
    let bottom_y = cmp::min(max_tile, (pane.y + offset) as u32 + 1);

    let top = XYZ {
        x: top_x as u32,
        y: top_y as u32,
        z: pane.z,
    };
    let bottom = XYZ {
        x: bottom_x as u32,
        y: bottom_y as u32,
        z: pane.z,
    };

    Plot {
        top: xyz_to_coordinate(&top),
        bottom: xyz_to_coordinate(&bottom),
    }
}
