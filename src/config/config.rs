//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml::{Value, Parser};
use std::io::prelude::*;
use std::fs::File;


pub trait Config<T> {
    fn from_config(config: &Value) -> Result<T, String>;
}

/// Load and parse the config file into Toml table structure.
/// If a file cannot be found are cannot parsed, return None.
pub fn read_config(path: &str) -> Result<Value, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_)  => {
            return Err("Could not find config file!".to_string());
        }
    };
    let mut config_toml = String::new();
    if let Err(err) = file.read_to_string(&mut config_toml) {
        return Err(format!("Error while reading config: [{}]", err));
    };

    parse_config(config_toml, path)
}

pub fn parse_config(config_toml: String, path: &str) -> Result<Value, String> {
    let mut parser = Parser::new(&config_toml);
    let toml = parser.parse();
    if toml.is_none() {
        let mut errors = Vec::new();
        for err in &parser.errors {
            let (loline, locol) = parser.to_linecol(err.lo);
            let (hiline, hicol) = parser.to_linecol(err.hi);
            errors.push(format!("{}:{}:{}-{}:{} error: {}",
                     path, loline, locol, hiline, hicol, err.desc));
        }
        return Err(errors.join("\n"));
    }

    Ok(Value::Table(toml.unwrap()))
 }

#[test]
fn test_parse_config() {
    let config = match read_config("src/test/example.cfg").unwrap() {
        Value::Table(table) => table,
        _ => panic!("Unexpected Value type")
    };
    println!("{:#?}", config);
    let expected = r#"{
    "cache": Table(
        {
            "strategy": String(
                "none"
            )
        }
    ),
    "datasource": Table(
        {
            "type": String(
                "postgis"
            ),
            "url": String(
                "postgresql://pi@localhost/natural_earth_vectors"
            )
        }
    ),
    "grid": Table(
        {
            "predefined": String(
                "web_mercator"
            )
        }
    ),
    "layer": Array(
        [
            Table(
                {
                    "fid_field": String(
                        "id"
                    ),
                    "geometry_field": String(
                        "wkb_geometry"
                    ),
                    "geometry_type": String(
                        "POINT"
                    ),
                    "name": String(
                        "points"
                    ),
                    "query": String(
                        "SELECT name,wkb_geometry FROM ne_10m_populated_places"
                    ),
                    "query_limit": Integer(
                        100
                    ),
                    "table_name": String(
                        "ne_10m_populated_places"
                    )
                }
            )
        ]
    ),
    "services": Table(
        {
            "mvt": Boolean(
                true
            )
        }
    ),
    "topics": Table(
        {
            "places": Array(
                [
                    String(
                        "points"
                    )
                ]
            )
        }
    ),
    "webserver": Table(
        {
            "bind": String(
                "0.0.0.0"
            ),
            "mapviewer": Boolean(
                true
            ),
            "port": Integer(
                8080
            ),
            "threads": Integer(
                4
            )
        }
    )
}"#;
    assert_eq!(expected, &*format!("{:#?}", config));

    for (key, value) in &config {
        println!("{}: \"{}\"", key, value);
    }

    assert!(config.contains_key("datasource"));

    let dsconfig = match config.get("datasource").unwrap() {
        &Value::Table(ref table) => table,
        _ => panic!("Unexpected Value type")
    };
    assert_eq!(format!("{}", dsconfig.get("type").unwrap()), "\"postgis\"");
}

#[test]
fn test_parse_error() {
    let config = read_config("src/config/mod.rs");
    assert_eq!("src/config/mod.rs:0:0-0:0 error: expected a key but found an empty string", config.err().unwrap());

    let config = read_config("wrongfile");
    assert_eq!("Could not find config file!", config.err().unwrap());
}