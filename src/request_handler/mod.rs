use hyper::server::{Request, Response, Service};
use config_loader::server_block::{AccumulatedServerBlock, UpstreamOption};
use hyper::header::Host;
use hyper::{StatusCode, Error, Client, Method};
use hyper::client;
use futures::{future, Future};
use tokio_core::reactor;
use std::io::{Read};
use response;
use fs_wrapper;

pub struct UpstreamRate<'a> {
    pub byte_matches: i32,
    pub upstream_option: &'a UpstreamOption
}

#[derive(Debug)]
pub struct AsyncHandler {
    pub accumulated_server_block: AccumulatedServerBlock,
    pub hyper_client: Client<client::HttpConnector>,
    pub client_core: reactor::Core
}

impl Service for AsyncHandler {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, mut req: Request) -> Self::Future {
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
                    let custom_path = best_upstream.upstream_option.destination_path.clone() + custom_end_path[1];
                    let absolute_path = best_upstream.upstream_option.target_host.clone() + custom_path.as_str();
                    println!("{:?}", absolute_path);

                    forward_request(
                        &self.hyper_client,
                        absolute_path,
                        &mut res,
                        &req
                    );
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

fn forward_request (
    hyper_client: &Client<client::HttpConnector>,
    path: String,
    response: &mut Response,
    original_request: &Request
) {
    let uri = path.parse().unwrap();

    // TODO: We need a type matcher here to take the referenced method and return non-ref as
    // this is a ref original_request.method().
    // We also need to read the request body, and do some mut header work like so
    // let mut request_headers = request.headers_mut();
    // request_headers = original_request.headers_mut();
    println!("Where we going {:?}", uri);
    let request: Request = Request::new(Method::Get, uri);

    let work = hyper_client.request(request).and_then(|res| {
        println!("Does the request return??");
        Ok(response.set_body(res.body()))
        // res.body().collect()

        // for_each(|chunk| {
        //     response.set_body(chunk)
        // })

        // .for_each(|chunk| {
        //        response.set_body(chunk);
        // })
    });

    reactor::Core::new().unwrap().run(work).unwrap();
}
