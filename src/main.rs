extern crate chrono;
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
use chrono::DateTime;

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
    let user_name = paramters.value_of("user_name").unwrap();

    let mut query_statement = String::from("SELECT TIMESTAMP, RAW_INTENSITY, STEPS, HEART_RATE FROM PEBBLE_HEALTH_ACTIVITY_SAMPLE WHERE DEVICE_ID = (?) AND USER_ID = (?)");
    let mut query_paramters = vec![device_id.to_owned(), user_id.to_owned()];
    if let Some(start_time) = paramters.value_of("start_time") {
        let start_time = match DateTime::parse_from_str(start_time, "%F") {
            Ok(d) => d,
            Err(err) => {
                return error!(
                    "Start time is invalid, format must be %Y-%m-%d (padded with zeros): {}",
                    err
                )
            }
        };
        query_statement.push_str(" AND TIMESTAMP > (?)");
        query_paramters.push(start_time.to_string());
    };
    if let Some(end_time) = paramters.value_of("end_time") {
        let end_time = match DateTime::parse_from_str(end_time, "%F") {
            Ok(d) => d,
            Err(err) => {
                return error!(
                    "End time is invalid, format must be %Y-%m-%d (padded with zeros): {}",
                    err
                )
            }
        };
        query_statement.push_str(" AND TIMESTAMP < (?)");
        query_paramters.push(end_time.to_string());
    };

    let connection = match Connection::open(input_file) {
        Ok(conn) => conn,
        Err(err) => return error!("Error opening database file: {}", err),
    };
    let mut statement = match connection.prepare(&query_statement) {
        Ok(stmt) => stmt,
        Err(err) => return error!("Error preparing the query statement: {}", err),
    };
    let query_paramters: Vec<&rusqlite::types::ToSql> = query_paramters
        .iter()
        .map(|p| p as &rusqlite::types::ToSql)
        .collect();
    let rows = match statement.query_map(query_paramters.as_slice(), |row| Entry {
        timestamp: row.get(0),
        raw_intensity: row.get(1),
        steps: row.get(2),
        heart_rate: row.get(3),
    }) {
        Ok(rows) => rows,
        Err(error) => return error!("Error querying database: {}", error),
    };

    let client = reqwest::Client::new();
    for chunk in &rows.chunks(100) {
        let mut data = String::new();
        for result in chunk {
            let row = match result {
                Ok(r) => r,
                Err(err) => return error!("Error reading database row: {}", err),
            };
            match write!(
                data,
                "raw_intensity,person={user_name} value={raw_intensity} {timestamp}000000000\n\
                 steps,person={user_name} value={steps} {timestamp}000000000\n\
                 heart_rate,person={user_name} value={heart_rate} {timestamp}000000000\n",
                raw_intensity = row.raw_intensity,
                steps = row.steps,
                heart_rate = row.heart_rate,
                timestamp = row.timestamp,
                user_name = user_name
            ) {
                Ok(_) => (),
                Err(err) => return error!("Error formatting data: {}", err),
            };
        }
        match client
            .post(
                format!(
                    "http://{host}:{port}/write?db={database}",
                    host = host,
                    port = port,
                    database = database
                ).as_str(),
            )
            .body(data)
            .send()
        {
            Ok(mut resp) => if !resp.status().is_success() {
                return error!(
                    "Error writing to database: {}",
                    resp.text()
                        .unwrap_or("Server did not return error".to_string())
                );
            },
            Err(err) => return error!("Error connecting to InfluxDB database: {}", err),
        };
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
            Arg::with_name("start_time")
                .short("s")
                .long("start_time")
                .help("Starting time of imported data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("end_time")
                .short("e")
                .long("end_time")
                .help("Ending time of imported data")
                .takes_value(true),
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
        .arg(
            Arg::with_name("user_name")
                .short("n")
                .long("user_name")
                .help("The user name in InfluxDB")
                .required(true)
                .takes_value(true),
        )
}
