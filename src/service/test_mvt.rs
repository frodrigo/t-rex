//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::PostgisInput;
use core::grid::Grid;
use core::layer::Layer;
use core::Config;
use cache::{Tilecache,Nocache};
use service::mvt::{Tileset, MvtService};


#[test]
#[ignore]
pub fn test_tile_query() {
    use std::env;

    let pg: PostgisInput = match env::var("DBCONN") {
        Result::Ok(val) => Some(PostgisInput::new(&val).connected()),
        Result::Err(_) => { panic!("DBCONN undefined") }
    }.unwrap();
    let grid = Grid::web_mercator();
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    layer.query_limit = Some(1);
    let tileset = Tileset{name: "points".to_string(), layers: vec![layer]};
    let mut service = MvtService {input: pg, grid: grid,
                              tilesets: vec![tileset], cache: Tilecache::Nocache(Nocache)};
    service.prepare_feature_queries();

    let mvt_tile = service.tile("points", 33, 41, 6);
    println!("{:#?}", mvt_tile);
    let expected = r#"Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some("points"),
            features: [
                Tile_Feature {
                    id: None,
                    tags: [
                        0,
                        0,
                        1,
                        1,
                        2,
                        2,
                        3,
                        3
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        2504,
                        3390
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            keys: [
                "fid",
                "scalerank",
                "name",
                "pop_max"
            ],
            values: [
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        106
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        10
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: Some("Delemont"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        11315
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell {
                value: 0
            }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell {
        value: 0
    }
}"#;
    assert_eq!(expected, &*format!("{:#?}", mvt_tile));
}

#[test]
pub fn test_mvt_metadata() {
    use core::read_config;

    let config = read_config("src/test/example.cfg").unwrap();
    let service = MvtService::from_config(&config).unwrap();

    let metadata = format!("{:#}", service.get_mvt_metadata().unwrap());
    let expected = r#"{
  "tilesets": [
    {
      "layers": [
        {
          "geometry_type": "POINT",
          "name": "points"
        },
        {
          "geometry_type": "POLYGON",
          "name": "buildings"
        },
        {
          "geometry_type": "POLYGON",
          "name": "admin_0_countries"
        }
      ],
      "name": "osm",
      "supported": true,
      "tilejson": "osm.json",
      "tileurl": "/osm/{z}/{x}/{y}.pbf"
    }
  ]
}"#;
    println!("{}", metadata);
    assert_eq!(metadata, expected);
}

#[test]
#[ignore]
pub fn test_tilejson() {
    use core::read_config;
    use std::env;

    if env::var("DBCONN").is_err() {
        panic!("DBCONN undefined");
    }

    let config = read_config("src/test/example.cfg").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    service.prepare_feature_queries();
    let metadata = format!("{:#}", service.get_tilejson("http://127.0.0.1", "osm").unwrap());
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": "[-180.0,-90.0,180.0,90.0]",
  "center": "[0.0, 0.0, 2]",
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "maxzoom": 14,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "tiles": [
    "http://127.0.0.1/osm/{z}/{x}/{y}.pbf"
  ],
  "vector_layers": [
    {
      "description": "",
      "fields": {
        "fid": "",
        "name": "",
        "pop_max": "",
        "scalerank": ""
      },
      "id": "points",
      "maxzoom": 99,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {},
      "id": "buildings",
      "maxzoom": 99,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {
        "fid": "",
        "iso_a3": "",
        "name": ""
      },
      "id": "admin_0_countries",
      "maxzoom": 99,
      "minzoom": 0
    }
  ],
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}

#[test]
pub fn test_stylejson() {
    use core::read_config;

    let config = read_config("src/test/example.cfg").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    let json = format!("{:#}", service.get_stylejson("http://127.0.0.1", "osm").unwrap());
    println!("{}", json);
    let expected= r#"
  "name": "t-rex",
  "sources": {
    "osm": {
      "type": "vector",
      "url": "http://127.0.0.1/osm.json"
    }
  },
  "version": 8
"#;
    assert!(json.contains(expected));
    let expected= r#"
  "layers": [
    {
      "id": "background_",
      "paint": {
        "background-color": "rgba(255, 255, 255, 1)"
      },
      "type": "background"
    },
    {
      "id": "points","#;
    assert!(json.contains(expected));

    let expected= r##"
      "paint": {
        "fill-color": "#d8e8c8",
        "fill-opacity": 0.5
      },"##;
    assert!(json.contains(expected));

    let expected= r#"
      "id": "buildings","#;
    assert!(json.contains(expected));
}

#[test]
#[ignore]
pub fn test_mbtiles_metadata() {
    use core::read_config;
    use std::env;

    if env::var("DBCONN").is_err() {
        panic!("DBCONN undefined");
    }

    let config = read_config("src/test/example.cfg").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    let metadata = format!("{:#}", service.get_mbtiles_metadata("osm").unwrap());
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": "[-180.0,-90.0,180.0,90.0]",
  "center": "[0.0, 0.0, 2]",
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "json": "{\"Layer\":[{\"description\":\"\",\"fields\":{\"fid\":\"\",\"name\":\"\",\"pop_max\":\"\",\"scalerank\":\"\"},\"id\":\"points\",\"name\":\"points\",\"properties\":{\"buffer-size\":0,\"maxzoom\":99,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"name\":\"buildings\",\"properties\":{\"buffer-size\":0,\"maxzoom\":99,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{\"fid\":\"\",\"iso_a3\":\"\",\"name\":\"\"},\"id\":\"admin_0_countries\",\"name\":\"admin_0_countries\",\"properties\":{\"buffer-size\":0,\"maxzoom\":99,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"}],\"vector_layers\":[{\"description\":\"\",\"fields\":{\"fid\":\"\",\"name\":\"\",\"pop_max\":\"\",\"scalerank\":\"\"},\"id\":\"points\",\"maxzoom\":99,\"minzoom\":0},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"maxzoom\":99,\"minzoom\":0},{\"description\":\"\",\"fields\":{\"fid\":\"\",\"iso_a3\":\"\",\"name\":\"\"},\"id\":\"admin_0_countries\",\"maxzoom\":99,\"minzoom\":0}]}",
  "maxzoom": 14,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}

#[test]
pub fn test_gen_config() {
    let expected = r#"# t-rex configuration

[service.mvt]
viewer = true

[datasource]
type = "postgis"
# Connection specification (https://github.com/sfackler/rust-postgres#connecting)
url = "postgresql://user:pass@host/database"

[grid]
# Predefined grids: web_mercator, wgs84
predefined = "web_mercator"

[[tileset]]
name = "points"

[[tileset.layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#fid_field = "id"
#simplify = true
#buffer-size = 10
#[[tileset.layer.query]]
#minzoom = 0
#maxzoom = 22
#sql = "SELECT name,wkb_geometry FROM mytable"

#[cache.file]
#base = "/tmp/mvtcache"
"#;
    println!("{}", &MvtService::gen_config());
    assert_eq!(expected, &MvtService::gen_config());
}
