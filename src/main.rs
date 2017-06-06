extern crate serde;
extern crate serde_json;
extern crate hyper;

#[macro_use]
extern crate serde_derive;

use std::io::{Read, Write, BufReader, BufRead, copy};
use std::net::{TcpStream};
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
    println!("{:?}", hostname);

    let path = match uri {
        AbsolutePath(x) => x,
        _ => "".to_string()
    };

    // Now that we know the domain, we perform a find over the accumulated server block on Host to get
    // the correct source path
    let mut iter = config.blocks.iter();
    let block_match = iter.find(|&b| b.host == hostname.as_str()).unwrap();

    println!("{:?}", block_match.base);
    println!("{:?}", block_match.source);

    // This needs to consider base path
    match path_match(path) {
        Ok(file) => {
            let is_file = file.metadata().unwrap().is_file();
            match is_file {
                true => {
                    let mut buf_reader = BufReader::new(file);
                    copy(&mut buf_reader, &mut res.start().unwrap()).unwrap();
                }
                false => {
                    *res.status_mut() = StatusCode::NotFound
                }
            }
        },
        Err(_) => {
            *res.status_mut() = StatusCode::NotFound
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
                Server::http(bind_address).unwrap().handle(move |mut req: Request, mut res: Response| {
                    request_handler(req, res, &block)
                }).unwrap();
                // let listener = TcpListener::bind(&bind_address).unwrap();
                // println!("Server listening on Port {}", bind_address);
                //
                // for stream in listener.incoming() {
                //     match stream {
                //         Ok(stream) => {
                //             handle_client(stream, &block);
                //         }
                //         Err(e) =>  {
                //             println!("Connection failed! What to do... {:?}", e);
                //         }
                //     }
                // }
            }
        ).unwrap());
    }

    for child in children {
        let _ = child.join();
    }
}

fn handle_client<'a>(stream: TcpStream, config: &AccumulatedServerBlock) {
    let mut reader = BufReader::new(stream);

    // This block creates a scope such that we can borrow from reader
    let path = {
        let mut line_iterator = reader.by_ref().lines();

        let first_line_opt = match line_iterator.next() {
            Some(line) => {
                match line {
                    Ok(target_line) => Some(target_line),
                    // Maybe We want to panic here
                    Err(_) => None
                }
            }
            None => {
                println!("No more lines :)");
                None
            }
        };

        let first_line = match first_line_opt {
            Some(line) => line,
            None => "".to_string()
        };

        // Just print the remaining lines for now
        for line in line_iterator {
            match line {
                Ok(line_val) => {
                    println!("The line is ---- {:?}", line_val);
                    if line_val == "" {
                        break;
                    }
                }
                Err(e) => {
                    println!("Line iterator error {:?}", e)
                }
            }
        }

        first_line
    };

    let split = path.split(" ");
    let vec: Vec<&str> = split.collect();

    let mut target_file = vec[1];
    if target_file == "/" {
        target_file = "/index"
    }

    println!("Target file ---- {:?}", target_file);
    send_response(reader.into_inner(), target_file);
}

fn send_response(mut stream: TcpStream, target_file: &str) {
    let target_path = "demo_site".to_string() + target_file + ".html";

    match File::open(target_path) {
        Ok(file) => {
            match stream.write("HTTP/1.1 OK\n\n".to_string().as_bytes()) {
                Ok(_) => {
                    let mut buf_reader = BufReader::new(file);
                    let lines = buf_reader.by_ref().lines();

                    for line in lines {
                        match line {
                            Ok(line_val) => {
                                println!("The response line is ---- {:?}", line_val);
                                let _ = stream.write(line_val.as_bytes());
                            },
                            Err(_) => panic!("Error writing file to res")
                        }
                    }
                },
                Err(_) => panic!("Failed to write to response")
            };
        }
        _ => {
            let response = response::not_found();
            let _ = stream.write_all(response.as_bytes());
        }
    }
}
