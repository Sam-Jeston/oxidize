use std::io::{Read, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::fs::File;

mod responses;

fn main() {
    // Unwrap seems a sensible choice here, as a panic is appropriate if the port is not
    // available
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on Port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) =>  {
                println!("Connection failed! What to do... {:?}", e);
            }
        }
    }
}

fn handle_client(stream : TcpStream) {
    let mut reader = BufReader::new(stream);

    // This block creates a scope such that we can borrow from reader
    let path = {
        let mut line_iterator = reader.by_ref().lines();
        let first_line = line_iterator.next().unwrap().unwrap();
        println!("The first line is ---- {:?}", first_line);

        // Just print the remaining lines for now
        for line in line_iterator {
            let line_val = line.unwrap();
            println!("The line is ---- {:?}", line_val);
            if line_val == "" {
                break;
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
    // TODO: Learn more about the questionmark operator. I understand it is error handling sugar
    // and unwrap is lazyness, but matching types is cumbersome
    let target_path = "demo_site".to_string() + target_file + ".html";

    match File::open(target_path) {
        Ok(file) => {
            stream.write("HTTP/1.1 OK\n\n".to_string().as_bytes()).unwrap();
            let mut buf_reader = BufReader::new(file);
            let lines = buf_reader.by_ref().lines();

            for line in lines {
                let line_val = line.unwrap();
                println!("The response line is ---- {:?}", line_val);
                stream.write(line_val.as_bytes()).unwrap();
            }
        }
        _ => {
            let response = responses::not_found();
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}