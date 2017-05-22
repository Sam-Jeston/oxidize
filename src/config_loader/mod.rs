use yaml_rust::{YamlLoader, Yaml};
use std::env;
use std::fs::File;
use std::io::prelude::*;

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

fn yaml_parser(yaml: String) -> Vec<Yaml> {
    let yaml_str_ref = yaml.as_str();
    YamlLoader::load_from_str(yaml_str_ref).unwrap()
}

pub fn load () -> Vec<Yaml> {
    let config_key = "OXIDIZE_CONFIG";
    match env::var_os(config_key) {
        Some(val) => {
            let file_string = read_file(val.into_string().unwrap());
            yaml_parser(file_string)
        }
        None => {
            let file_string = read_file("src/config_loader/default.yml".to_string());
            yaml_parser(file_string)
        }
    }
}
