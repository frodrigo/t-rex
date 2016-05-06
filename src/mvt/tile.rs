use core::layer::Layer;
use core::feature::Feature;
use core::grid::Extent;
use core::geom::GeometryType;
use core::geom;
use core::screen;
use mvt::vector_tile;
use mvt::geom_to_proto::{EncodableGeom,CommandSequence};


pub struct Tile<'a> {
    pub mvt_tile: vector_tile::Tile,
    tile_size: u32,
    extent: &'a Extent,
    feature_id: u64,
}

// --- conversion of geometries into screen coordinates

trait ScreenGeom<T> {
    fn from_geom(extent: &Extent, tile_size: u32, geom: T) -> Self;
}

impl ScreenGeom<geom::Point> for screen::Point {
    fn from_geom(extent: &Extent, tile_size: u32, point: geom::Point) -> Self {
        let x_span = extent.maxx - extent.minx;
        let y_span = extent.maxy - extent.miny;
        screen::Point {
            x: ((point.x-extent.minx) * tile_size as f64 / x_span) as i32,
            y: ((point.y-extent.miny) * tile_size as f64 / y_span) as i32
        }
    }
}


#[test]
fn test_point_to_screen_coords() {
    //let zh_mercator = geom::Point::new(949398.0, 6002729.0);
    let zh_mercator = geom::Point::new(960000.0, 6002729.0);
    //let zh_wgs84 = postgis::Point::<WGS84>::new(47.3703149, 8.5285874);
    let tile_extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    let screen_pt = screen::Point::from_geom(&tile_extent, 4096, zh_mercator);
    assert_eq!(screen_pt, screen::Point { x: 245, y: 3131 });
    //assert_eq!(screen_pt.encode().0, &[9,490,6262]);
}


// --- Tile creation functions

impl<'a> Tile<'a> {
    pub fn new(extent: &Extent, tile_size: u32) -> Tile {
        let mvt_tile = vector_tile::Tile::new();
        Tile {mvt_tile: mvt_tile, tile_size: tile_size, extent: extent, feature_id: 0 }
    }

    pub fn new_layer(&mut self, layer: Layer) -> vector_tile::Tile_Layer {
        vector_tile::Tile_Layer::new()
    }

    pub fn new_feature(&self, feature: Feature) -> vector_tile::Tile_Feature {
        vector_tile::Tile_Feature::new()
    }

    pub fn encode_geom(&self, geom: geom::GeometryType) -> CommandSequence {
        let screen_geom = match geom {
            GeometryType::Point(p) => screen::Point::from_geom(&self.extent, self.tile_size, p),
            _ => panic!("Geometry type not implemented yet")
        };
        screen_geom.encode()
    }

    fn add_feature_attribute(mvt_layer: &mut vector_tile::Tile_Layer,
                     mvt_feature: &mut vector_tile::Tile_Feature,
                     key: String, mvt_value: vector_tile::Tile_Value) {
        let keyentry = mvt_layer.get_keys().iter().position(|k| *k == key);
        // Optimization: maintain a hash table with key/index pairs
        let keyidx = match keyentry {
            None => {
                mvt_layer.mut_keys().push(key);
                mvt_layer.get_keys().len()-1
            },
            Some(idx) => idx
        };
        mvt_feature.mut_tags().push(keyidx as u32);

        let valentry = mvt_layer.get_values().iter().position(|v| *v == mvt_value);
        // Optimization: maintain a hash table with value/index pairs
        let validx = match valentry {
            None => {
                mvt_layer.mut_values().push(mvt_value);
                mvt_layer.get_values().len()-1
            },
            Some(idx) => idx
        };
        mvt_feature.mut_tags().push(validx as u32);
    }

    pub fn add_layer(&mut self, mvt_layer: vector_tile::Tile_Layer) {

    }
}

#[test]
fn test_tile_values() {
    let mut value = vector_tile::Tile_Value::new();
    assert_eq!(value, Default::default());
    assert!(!value.has_string_value());
    value.set_string_value(String::from("Hello, world!"));
    println!("{:?}", value);
    assert!(value.has_string_value());
    assert_eq!(value.get_string_value(), String::from("Hello, world!"));
}

#[test]
fn test_read_pbf_file() {
    use std::fs::File;
    use std::io::BufReader;
    use protobuf::parse_from_reader;

    let f = File::open("src/test/tile.pbf").unwrap();
    // Command line decoding:
    // protoc --decode=vector_tile.Tile src/mvt/vector_tile.proto <src/test/tile.pbf
    let mut reader = BufReader::new(f);
    let tile = parse_from_reader::<vector_tile::Tile>(&mut reader).unwrap();
    println!("{:#?}", tile);
    let ref layer = tile.get_layers()[0];
    assert_eq!(layer.get_name(), "roads");
    let ref feature = layer.get_features()[1];
    assert_eq!(feature.get_field_type(), vector_tile::Tile_GeomType::POLYGON);
    let ref geometry = feature.get_geometry();
    assert_eq!(geometry, &[9,8236,4926,34,9,24,37,21,10,7,4,19,15]);
}


#[test]
fn test_create_pbf() {
    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
    let mut mvt_tile = vector_tile::Tile::new();

    let mut mvt_layer = vector_tile::Tile_Layer::new();
    mvt_layer.set_version(2);
    mvt_layer.set_name(String::from("points"));
    mvt_layer.set_extent(4096);

    let mut mvt_feature = vector_tile::Tile_Feature::new();
    mvt_feature.set_id(1);
    mvt_feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    mvt_feature.set_geometry([9, 2410, 3080].to_vec());

    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("world"));
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("hello"), mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("world"));
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("h"), mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_double_value(1.23);
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("count"), mvt_value);

    mvt_layer.mut_features().push(mvt_feature);

    mvt_feature = vector_tile::Tile_Feature::new();
    mvt_feature.set_id(2);
    mvt_feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    mvt_feature.set_geometry([9, 2410, 3080].to_vec());

    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("again"));
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("hello"), mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_int_value(2);
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("count"), mvt_value);

    mvt_layer.mut_features().push(mvt_feature);

    mvt_tile.mut_layers().push(mvt_layer);
    println!("{:#?}", mvt_tile);
    let expected = "Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some(\"points\"),
            features: [
                Tile_Feature {
                    id: Some(
                        1
                    ),
                    tags: [
                        0,
                        0,
                        1,
                        0,
                        2,
                        1
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        2410,
                        3080
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Feature {
                    id: Some(
                        2
                    ),
                    tags: [
                        0,
                        2,
                        2,
                        3
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        2410,
                        3080
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                }
            ],
            keys: [
                \"hello\",
                \"h\",
                \"count\"
            ],
            values: [
                Tile_Value {
                    string_value: Some(\"world\"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        1.23
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Value {
                    string_value: Some(\"again\"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        2
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                }
            ],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell { value: 0 }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell { value: 0 }
}";
    assert_eq!(expected, &*format!("{:#?}", mvt_tile));
}