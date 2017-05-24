use std::env;
use std::fs::File;
use std::io::prelude::*;
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct ServerBlock {
    port: u32,
    source: String,
    root: String,
    base: String
}

fn read_file(file_path: String) -> String {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents
        }
        Err(e) => {
            panic!("Where art thou config! {:?}", e);
        }
    }
}

fn json_parser(json: String) -> Vec<ServerBlock> {
    let json_result: Vec<ServerBlock> = serde_json::from_str(json.as_str()).unwrap();

    for j in &json_result {
        println!("Our first server port is {} with base file of {}", j.port, j.base);
    }

    json_result
}

pub fn load () -> Vec<ServerBlock> {
    let config_key = "OXIDIZE_CONFIG";
    match env::var_os(config_key) {
        Some(val) => {
            let file_string = read_file(val.into_string().unwrap());
            json_parser(file_string)
        }
        None => {
            let file_string = read_file("src/config_loader/default.json".to_string());
            json_parser(file_string)
        }
    }
}
