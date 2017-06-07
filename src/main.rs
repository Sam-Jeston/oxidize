extern crate serde;
extern crate serde_json;
extern crate hyper;

#[macro_use]
extern crate serde_derive;

use std::io::{BufReader, copy};
use std::fs::{File};
use config_loader::server_block::{AccumulatedServerBlock};
use std::thread;
use hyper::server::{Server, Request, Response};
use hyper::status::StatusCode;
use hyper::uri::RequestUri::{AbsolutePath};
use hyper::header::Host;

mod response;
mod config_loader;

fn request_handler (req: Request, mut res: Response, config: &AccumulatedServerBlock) {
    let uri = req.uri;

    // We probably should match this Option, but will need to create a default Host header val
    let domain = req.headers.get::  <Host>().unwrap();
    let hostname = &domain.hostname;

    let mut path = match uri {
        AbsolutePath(x) => x,
        _ => "".to_string()
    };

    // Now that we know the domain, we perform a find over the accumulated server block on Host to get
    // the correct source path
    let mut iter = config.blocks.iter();
    let block_match = iter.find(|&b| b.host == hostname.as_str()).unwrap();

    if path == "/" {
        path = "/".to_string() + block_match.base.clone().as_str();
    }

    let absolute_path = block_match.source.clone() + path.as_str();

    // This needs to consider base path
    match path_match(absolute_path) {
        Ok(file) => {
            let is_file = file.metadata().unwrap().is_file();
            match is_file {
                true => {
                    let mut buf_reader = BufReader::new(file);
                    copy(&mut buf_reader, &mut res.start().unwrap()).unwrap();
                }
                false => {
                    *res.status_mut() = StatusCode::NotFound;
                }
            }
        },
        Err(_) => {
            *res.status_mut() = StatusCode::NotFound;
            let not_found = response::not_found();
            let response_bytes = not_found.as_bytes();
            res.send(response_bytes).unwrap();
        }
    }
}

/// Return a Result<file buffer> or Error(String) to the request handler based on the existence of
/// the file
fn path_match (path: String) -> Result<File, String> {
    match File::open(path) {
        Ok(file) => {
            Result::Ok(file)
        }
        _ => {
            println!("File does not exist");
            Result::Err("File does not exist".to_string())
        }
    }
}

fn main() {
    let accumulated_server_blocks = config_loader::load();

    let mut children = vec![];

    for block in accumulated_server_blocks {
        children.push(thread::Builder::new().name("Oxidize-Server-Port-".to_string() + block.port.to_string().as_str())
            .spawn(move || {
                let bind_address = "0.0.0.0:".to_string() + block.port.to_string().as_str();
                Server::http(bind_address).unwrap().handle(move |req: Request, res: Response| {
                    request_handler(req, res, &block)
                }).unwrap();
            }
        ).unwrap());
    }

    for child in children {
        let _ = child.join();
    }
}
