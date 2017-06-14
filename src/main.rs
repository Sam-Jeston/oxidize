extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate futures;

#[macro_use]
extern crate serde_derive;

use std::thread;
use hyper::server::{Http, Request, Response, Service};

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
                let bind_address = ("0.0.0.0:".to_string() + block.port.to_string().as_str()).parse().unwrap();
                let server = Http::new().bind(&bind_address, || Ok(request_handler::AsyncHandler)).unwrap();
                server.run().unwrap();
            }
        ).unwrap());
    }

    for child in children {
        let _ = child.join();
    }
}
