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
                let path = get_path(parse_request(&request)[0]);
                let _ = match path {
                    "/" => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n"),
                    _ => stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n"),
                };
                println!("Accepted new connection")
            }
            Err(e) => println!("Error {}", e),
        }
    }
}
