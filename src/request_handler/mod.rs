use hyper::server::{Request, Response};
use config_loader::server_block::{AccumulatedServerBlock};
use hyper::header::Host;
use hyper::uri::RequestUri::{AbsolutePath};
use std::io::{BufReader, copy};
use hyper::status::StatusCode;
use response;
use fs_wrapper;

pub fn handle (req: Request, mut res: Response, config: &AccumulatedServerBlock) {
    let uri = req.uri;

    // We probably should match this Option, but will need to create a default Host header val
    let domain = req.headers.get::<Host>().unwrap();
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
    match fs_wrapper::file_match(absolute_path) {
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
