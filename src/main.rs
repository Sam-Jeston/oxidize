extern crate serde;
extern crate serde_json;
extern crate hyper;

#[macro_use]
extern crate serde_derive;

use std::thread;
use hyper::server::{Server, Request, Response};

mod response;
mod config_loader;
mod request_handler;
mod fs_wrapper;

fn main() {
    let accumulated_server_blocks = config_loader::load();

    let mut children = vec![];

    for block in accumulated_server_blocks {
        children.push(thread::Builder::new().name("Oxidize-Server-Port-".to_string() + block.port.to_string().as_str())
            .spawn(move || {
                let bind_address = "0.0.0.0:".to_string() + block.port.to_string().as_str();
                Server::http(bind_address).unwrap().handle(move |req: Request, res: Response| {
                    request_handler::handle(req, res, &block)
                }).unwrap();
            }
        ).unwrap());
    }

    for child in children {
        let _ = child.join();
    }
}
