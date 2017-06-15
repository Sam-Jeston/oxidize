extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate futures;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;

use std::thread;
use hyper::server::{Http, Request, Response, Service};
use request_handler::AsyncHandler;
use tokio_core::reactor::Core;

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
                let server = Http::new().bind(&bind_address, move || {
                    // Instantiate our keep-alive client instance for proxy passing
                    let core = Core::new().unwrap();
                    let client = hyper::Client::configure().keep_alive(true).build(&core.handle());
                    let handler = AsyncHandler {
                        accumulated_server_block: block.clone(),
                        hyper_client: client,
                        handle: core.handle().clone()
                    };

                    Ok(handler)
                }).unwrap();
                server.run().unwrap();
            }
        ).unwrap());
    }

    for child in children {
        let _ = child.join();
    }
}
