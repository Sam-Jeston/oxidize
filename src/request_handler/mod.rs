use hyper::server::{Request, Response, Service};
use config_loader::server_block::{AccumulatedServerBlock, UpstreamOption};
use hyper::header::Host;
use hyper::{StatusCode, Error};
use futures::future;
use std::io::{Read};
use response;
use fs_wrapper;

pub struct UpstreamRate<'a> {
    pub byte_matches: i32,
    pub upstream_option: &'a UpstreamOption
}

#[derive(Debug, Clone)]
pub struct AsyncHandler {
    pub accumulated_server_block: AccumulatedServerBlock
}

impl Service for AsyncHandler {
    type Request = Request;
    type Response = Response;
    type Error = Error;

    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let mut res = Response::new();

        // We probably should match this Option, but will need to create a default Host header val
        let domain = req.headers().get::<Host>().unwrap();
        let hostname = &domain.hostname();

        // Now that we know the domain, we perform a find over the accumulated server block on Host to get
        // the correct source path
        let target_blocks = &self.accumulated_server_block.blocks.clone();
        let mut iter = target_blocks.iter();
        let ref block_match = iter.find(|&b| &b.host == &hostname.to_string()).unwrap();

        let block_match_base = block_match.base.clone();
        let block_match_str = block_match_base.as_str();
        let default_path = "/".to_string() + block_match_str;
        let mut path = req.uri().path();

        // We will use this for our upstreams, and as such need a copy of the original path
        let path_ref = path.clone();
        if path == "/" {
            path = default_path.as_str();
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

                if exact_match {
                    let custom_end_path: Vec<&str> = path_ref.split(best_upstream.upstream_option.source_path.as_str()).collect();
                    let absolute_path = best_upstream.upstream_option.destination_path.clone() + custom_end_path[1];

                    println!("{:?}", absolute_path);

                    serve_file(absolute_path, &mut res);
                } else {
                    let absolute_path = block_match.source.clone() + path;
                    serve_file(absolute_path, &mut res);
                }
            },
            None => {
                let absolute_path = block_match.source.clone() + path;
                serve_file(absolute_path, &mut res);
            }
        }

        future::ok(res)
    }
}

fn serve_file (absolute_path: String, response: &mut Response) {
    match fs_wrapper::file_match(absolute_path) {
        Ok(mut file) => {
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
                    response.set_body(response::not_found());
                }
            }
        },
        Err(_) => {
            response.set_status(StatusCode::NotFound);
            response.set_body(response::not_found());
        }
    }
}
