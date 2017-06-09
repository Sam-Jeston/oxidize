use std::env;
use std::io::prelude::*;
use serde_json;
use config_loader::server_block::{ServerBlock, AccumulatedServerBlock, accumlated_server_blocks};
use fs_wrapper;

pub mod server_block;

fn read_file(file_path: String) -> String {
    // I appreciate the FS::open wrapper looks pointless, but I may want to handle things in that mod
    // directly
    match fs_wrapper::file_match(file_path) {
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

pub fn load () -> Vec<AccumulatedServerBlock> {
    let config_key = "OXIDIZE_CONFIG";
    match env::var_os(config_key) {
        Some(val) => {
            let file_string = read_file(val.into_string().unwrap());
            let raw_blocks = json_parser(file_string);
            accumlated_server_blocks(raw_blocks)
        }
        None => {
            // TODO: This clearly not a long term solution
            let file_string = read_file("/home/dev/Projects/rust/oxidize/src/config_loader/default.json".to_string());
            let raw_blocks = json_parser(file_string);
            accumlated_server_blocks(raw_blocks)
        }
    }
}
