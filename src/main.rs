extern crate clap;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate rusqlite;

use clap::{App, Arg};

fn main() {
    let paramters = app().get_matches();
    let input_file = paramters.value_of("file").unwrap();
    let host = paramters.value_of("host").unwrap();
    let port = paramters.value_of("port").unwrap();
    let database = paramters.value_of("database").unwrap();
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
                .help("The Sqlite database")
                .required(true)
                .default("health.db")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .help("Host of the InfluxDB")
                .required(true)
                .default_value("localhost")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port of the InfluxDB")
                .required(true)
                .default_value("8086")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .help("The InfluxDB database")
                .required(true)
                .default_value("health")
                .takes_value(true),
        )
}
