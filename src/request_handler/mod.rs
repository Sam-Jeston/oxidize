use hyper::server::{Request, Response};
use config_loader::server_block::{AccumulatedServerBlock, UpstreamOption};
use hyper::header::Host;
use std::io::{BufReader, copy};
use hyper::{StatusCode};
use hyper::server::{Service};
use hyper::Error;
use futures::future;
use std::io::{Read};
use std::fs::File;
use response;
use fs_wrapper;

pub struct UpstreamRate<'a> {
    pub byte_matches: i32,
    pub upstream_option: &'a UpstreamOption
}

pub struct AsyncHandler;

impl Service for AsyncHandler {
    type Request = Request;
    type Response = Response;
    type Error = Error;

    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        let mut path = req.uri().path();

        // We probably should match this Option, but will need to create a default Host header val
        let domain = req.headers().get::<Host>().unwrap();
        let hostname = &domain.hostname();

        // Now that we know the domain, we perform a find over the accumulated server block on Host to get
        // the correct source path
        let mut iter = config.blocks.iter();
        let block_match = iter.find(|&b| b.host == hostname).unwrap();

        // We will use this for our upstreams, and as such need a copy of the original path
        let path_ref = path.clone();

        if path == "/" {
            path = ("/".to_string() + block_match.base.clone().as_str()).as_str();
        }

        // TODO: Here we will match against the Optinal upstream, and see if our path matches the absolute path
        // and will instead redirect to the new upstream
        match block_match.upstreams {
            Some(ref u) => {
                // Let's iterate each upstream, and then compare byte by byte, we will want to score
                // each entry on number of byte matches, and then once that is done, do a character
                // comparison to make sure we actually have a valid match
                let original_path_bytes = path_ref.as_bytes();

                let best_upstream: UpstreamRate = u.iter().map(|upstream| {
                    let upstream_bytes = upstream.source_path.as_bytes();

                    let mut matches: i32 = 0;
                    let original_len_ref = original_path_bytes.len();
                    for (i, b) in upstream_bytes.iter().enumerate() {
                        if original_len_ref <= i {
                            break;
                        }

                        if b == &original_path_bytes[i] {
                            matches = matches + 1;
                        } else {
                            break;
                        }
                    }

                    UpstreamRate { byte_matches: matches, upstream_option: upstream}
                }).max_by_key(|upstream| upstream.byte_matches).unwrap();

                println!("best upstream info. Matches: {:?}, path: {:?}", best_upstream.byte_matches, best_upstream.upstream_option.source_path);

                // Okay so now, lets split our strings on "/". Then see if we have any actual matches
                // on path words
                let upsteam_path_split: Vec<&str> = best_upstream.upstream_option.source_path.split("/").collect();
                let current_path_split: Vec<&str> = path_ref.split("/").collect();

                let upstream_len_ref = upsteam_path_split.len();

                let mut exact_match = false;
                for (i, path_word) in current_path_split.iter().enumerate() {
                    if upstream_len_ref <= i {
                        break;
                    }

                    if path_word == &upsteam_path_split[i] {
                        if upstream_len_ref == i + 1 {
                            exact_match = true;
                            break;
                        }
                    } else {
                        break;
                    }
                }

                println!("Exact match? {:?}", exact_match);

                if exact_match {
                    let custom_end_path: Vec<&str> = path_ref.split(best_upstream.upstream_option.source_path.as_str()).collect();
                    let absolute_path = best_upstream.upstream_option.destination_path.clone() + custom_end_path[1];

                    println!("{:?}", custom_end_path[1]);

                    let absolute_path = block_match.source.clone() + path;
                    serve_file(absolute_path, response);
                } else {
                    let absolute_path = block_match.source.clone() + path;
                    serve_file(absolute_path, response);
                }
            },
            None => {
                let absolute_path = block_match.source.clone() + path;
                serve_file(absolute_path, response);
            }
        }

        future::ok(response)
    }
}

fn serve_file (absolute_path: String, mut response: Response) {
    match fs_wrapper::file_match(absolute_path) {
        Ok(file) => {
            let is_file = file.metadata().unwrap().is_file();
            match is_file {
                true => {
                    let mut out = Vec::new();
                    if file.read_to_end(&mut out).is_ok() {
                        response.set_body(out);
                    }

                    // TODO: Buffer was a better approach, figure that out
                    // let mut buf_reader = BufReader::new(file);
                    // copy(&mut buf_reader, &mut res.start().unwrap()).unwrap();
                    // response.with_body(buf_reader);
                }
                false => {
                    response.set_status(StatusCode::NotFound);
                    let not_found = response::not_found();
                    let response_bytes = not_found.as_bytes();
                    response.set_body(response_bytes);
                }
            }
        },
        Err(_) => {
            response.set_status(StatusCode::NotFound);
            let not_found = response::not_found();
            let response_bytes = not_found.as_bytes();
            response.set_body(response_bytes);
        }
    }
}
