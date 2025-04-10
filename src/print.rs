/***********
 * Imports *
 ***********/

// Mine
use crate::utils;
// Other
use chrono::Utc;
use colored::Colorize;

/*************
 * Variables *
 *************/

// Time
const STRFTIME: &str = "%y-%m-%d %T";

// Literals
const PRINT_COMING: &str = "<<<";
const PRINT_GOING: &str = ">>>";
const PRINT_INFO: &str = "INFO";

/*************
 * Functions *
 *************/

pub fn print_info(data: &str) -> () {
    let now = chrono::Utc::now();
    println!(
        "[{}] [{}] {}",
        now.format(STRFTIME).to_string(),
        PRINT_INFO.yellow(),
        data,
    );
}

pub fn print_in(addr: &str, user_agent: &str) -> () {
    let now = chrono::Utc::now();
    println!(
        "[{}] {} {} - {}",
        now.format(STRFTIME).to_string(),
        PRINT_COMING.green(),
        addr,
        user_agent
    );
}

pub fn print_out_xyz(position: &utils::XYZ) -> () {
    let now = Utc::now();
    println!(
        "[{}] {} Tile - {}, {}, {}",
        now.format(STRFTIME).to_string(),
        PRINT_GOING.red(),
        position.x,
        position.y,
        position.z,
    );
}

pub fn print_out_plot(position: &utils::Plot) -> () {
    let now = Utc::now();
    println!(
        "[{}] {} JSON - {}, {}, {}, {}",
        now.format(STRFTIME).to_string(),
        PRINT_GOING.red(),
        position.top.lat,
        position.top.lon,
        position.bottom.lat,
        position.bottom.lon,
    );
}
