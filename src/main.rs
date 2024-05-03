// Uncomment this block to pass the first stage
use std::{
    // collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
};

fn parse_request(request: &str) -> Vec<&str> {
    request.split("\r\n").collect()
}
fn get_path(request_start_line: &str) -> &str {
    let req_arr: Vec<_> = request_start_line.split(" ").collect();
    req_arr[1]
}

fn make_response(randstr: &str) -> String {
    let content_length = randstr.len();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        content_length, randstr
    );
    response

    // response
}
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..]);
                let parsed_request = parse_request(&request);
                println!("parsed_request: {:?}", parsed_request);
                let path = get_path(parsed_request[0]);
                let _ = match path {
                    "/" => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n"),
                    "/user-agent" => {
                        let user_agent: Vec<_> = parsed_request[2].split(":").collect();
                        let user_agent = &user_agent[1].trim_start();
                        let response_str = make_response(user_agent);
                        let response = response_str.as_bytes();
                        stream.write_all(response)
                    }
                    _ => {
                        if path.starts_with("/echo/") {
                            let path_split_vec: Vec<_> = path.split("/").collect();
                            let random_string = path_split_vec[2];
                            let response_str = make_response(random_string);
                            let response = response_str.as_bytes();
                            stream.write_all(response)
                        } else {
                            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        }
                    }
                };
                println!("Accepted new connection")
            }
            Err(e) => println!("Error {}", e),
        }
    }
}
