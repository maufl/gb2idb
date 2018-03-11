#![feature(conservative_impl_trait)]

extern crate clap;
extern crate env_logger;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate rusqlite;

use clap::{App, Arg};
use rusqlite::Connection;
use itertools::Itertools;
use std::fmt::Write;

struct Entry {
    timestamp: u32,
    raw_intensity: u32,
    steps: u32,
    heart_rate: u32,
}

fn main() {
    env_logger::init();

    let paramters = app().get_matches();
    let input_file = paramters.value_of("file").unwrap();
    let user_id = paramters.value_of("user_id").unwrap();
    let device_id = paramters.value_of("device_id").unwrap();
    let host = paramters.value_of("host").unwrap();
    let port = paramters.value_of("port").unwrap();
    let database = paramters.value_of("database").unwrap();

    let connection = match Connection::open(input_file) {
        Ok(conn) => conn,
        Err(err) => return error!("Error opening database file: {}", err),
    };
    let mut statement = match 
        connection.prepare("SELECT TIMESTAMP, RAW_INTENSITY, STEPS, HEART_RATE FROM PEBBLE_HEALTH_ACTIVITY_SAMPLE WHERE DEVICE_ID = (?) AND USER_ID = (?)") {
            Ok(stmt) => stmt,
            Err(err) => return error!("Error preparing the query statement: {}", err)
        };
    let rows = match statement.query_map(&[&device_id, &user_id], |row| Entry {
        timestamp: row.get(0),
        raw_intensity: row.get(1),
        steps: row.get(2),
        heart_rate: row.get(3),
    }) {
        Ok(rows) => rows,
        Err(error) => return error!("Error querying database: {}", error),
    };
    for chunk in &rows.chunks(100) {
        let mut data = String::new();
        for result in chunk {
            let row = match result {
                Ok(r) => r,
                Err(err) => return error!("Error reading database row: {}", err),
            };
            match write!(
                data,
                "raw_intensity,person=felix value={raw_intensity} {timestamp}000000000\n\
                 steps,person=felix value={steps} {timestamp}000000000\n\
                 heart_rate,person=felix value={heart_rate} {timestamp}000000000\n",
                raw_intensity = row.raw_intensity,
                steps = row.steps,
                heart_rate = row.heart_rate,
                timestamp = row.timestamp
            ) {
                Ok(_) => (),
                Err(err) => return error!("Error formatting data: {}", err),
            };
        }
    }
}

fn app() -> App<'static, 'static> {
    App::new("gb2idb")
        .version("0.1")
        .author("Felix Konstantin Maurer <github@maufl.de>")
        .about(
            "Utility to read exported health data from Gadgetbridge and write it into an InfluxDB.",
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("The SQLite database")
                .default_value("health.db"),
        )
        .arg(
            Arg::with_name("user_id")
                .short("u")
                .long("user_id")
                .help("The user id")
                .default_value("1"),
        )
        .arg(
            Arg::with_name("device_id")
                .short("g")
                .long("device_id")
                .help("The device id")
                .default_value("1"),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .help("Host of the InfluxDB")
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port of the InfluxDB")
                .default_value("8086"),
        )
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .help("The InfluxDB database")
                .default_value("health"),
        )
}
