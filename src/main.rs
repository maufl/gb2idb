extern crate clap;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate rusqlite;

use clap::{App, Arg};

fn main() {
    let paramters = app().get_matches();
    let input_file = paramters.value_of("file").unwrap();
    println!("Hello, world!");
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
                .takes_value(true),
        )
}
