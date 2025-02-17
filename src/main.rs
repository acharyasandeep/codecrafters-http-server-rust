// Uncomment this block to pass the first stage
use std::{
    // collections::HashMap,
    collections::HashMap,
    env,
    fs,
    io,
    io::{BufRead, BufReader, Error, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    str,
    thread,
};

use bytes::buf;
#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    protocol: String,
    http_headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn new(mut stream: &TcpStream) -> Self {
        // let mut buffer = [0; 1024];
        // let _ = stream.read(&mut buffer);
        // let buffer = String::from_utf8_lossy(&buffer);
        // println!("Stream is: {:?}", buffer);
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_line = String::new();

        let _ = buf_reader.read_line(&mut request_line);
        let request_line = request_line.trim();
        let mut request_line_split = request_line.split(" ");
        let method = request_line_split.next().unwrap_or_else(|| "GET");
        let path = request_line_split.next().unwrap_or_else(|| "/");
        let protocol = request_line_split.next().unwrap_or_else(|| "HTTP/1.1");

        let mut http_headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            let _ = buf_reader.read_line(&mut line);
            let line = line.trim();
            println!("line is: {}", line);
            if line == "" {
                break;
            }
            let Some((key, value)) = line.split_once(": ") else {
                panic!("Cannot split {:?}", line);
            };
            http_headers.insert(key.to_string(), value.to_string());
        }

        let default_buffer = String::from("0");
        let buffer_size = http_headers
            .get("Content-Length")
            .unwrap_or(&default_buffer)
            .parse::<usize>()
            .unwrap_or_else(|_| 0);

        println!("buf_reader is : {:?} ", buf_reader);

        let mut buf = vec![0; buffer_size];
        let _ = buf_reader.read_exact(&mut buf);

        let body = String::from_utf8_lossy(&buf).to_string();

        Request {
            method: method.to_string(),
            path: path.to_string(),
            protocol: protocol.to_string(),
            http_headers,
            body,
        }
    }
}

// fn parse_request(request: &str) -> Vec<&str> {
//     request.split("\r\n").collect()
// }
// fn get_path(request_start_line: &str) -> &str {
//     let req_arr: Vec<_> = request_start_line.split(" ").collect();
//     req_arr[1]
// }

fn make_response(randstr: &str, content_type: &str, response_code: &str) -> String {
    let content_length = randstr.len();
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        response_code, content_type, content_length, randstr
    );
    response

    // response
}

fn handle_request_helper(stream: Result<TcpStream, Error>, directory: String) {
    match stream {
        Ok(mut stream) => {
            let request = Request::new(&stream);
            println!("request is: {:?}", request);
            handle_request(request, &mut stream, directory);
        }
        Err(e) => println!("Error {}", e),
    }
}

fn handle_request(request: Request, stream: &mut TcpStream, directory: String) {
    let path = request.path;
    println!("path is :{}", path);
    let method = request.method;
    let _ = match method.as_str() {
        "GET" => match path.as_str() {
            "/" => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n"),
            "/user-agent" => {
                let default_user_agent = String::from("curl/8.4.0");
                let user_agent = request
                    .http_headers
                    .get("User-Agent")
                    .unwrap_or_else(|| &default_user_agent);
                let response_str = make_response(user_agent.as_str(), "text/plain", "200 OK");
                let response = response_str.as_bytes();
                stream.write_all(response)
            }
            _ => {
                if path.starts_with("/echo/") {
                    let path_split_vec: Vec<_> = path.split("/").collect();
                    let random_string = path_split_vec[2];
                    let response_str = make_response(random_string, "text/plain", "200 OK");
                    let response = response_str.as_bytes();
                    stream.write_all(response)
                } else if path.starts_with("/files/") {
                    let path_split_vec = path.split_once("/files/");
                    if let Some((_, filename)) = path_split_vec {
                        let file_path = directory + "/" + filename;
                        if Path::new(&file_path).exists() {
                            let content = fs::read_to_string(file_path);
                            let content = content.unwrap_or_else(|_| String::from(""));

                            let response = make_response(
                                content.as_str(),
                                "application/octet-stream",
                                "200 OK",
                            );
                            let _ = stream.write_all(response.as_bytes());
                            return;
                        } else {
                            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                            return;
                        }
                    }
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                } else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                }
            }
        },
        "POST" => match path.as_str() {
            _ => {
                if path.starts_with("/files/") {
                    let path_split_vec = path.split_once("/files/");

                    if let Some((_, filename)) = path_split_vec {
                        let file_path = directory + "/" + filename;
                        println!("file_path: {}", file_path);
                        let body = request.body;
                        let mut file = fs::File::create(file_path).unwrap();
                        let _ = file.write_all(body.as_bytes());

                        let response = make_response("", "text/plain", "201 Created");
                        let _ = stream.write_all(response.as_bytes());
                        return;
                    } else {
                        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                        return;
                    }
                }
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
            }
        },

        _ => stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n"),
    };
    println!("Accepted new connection")
}
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);
    let mut directory = String::from("");
    if args.len() == 3 {
        if args[1] == "--directory" {
            directory = args[2].clone();
        }
    }
    if directory.ends_with("/") {
        directory = directory[0..directory.len() - 1].to_string();
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let dir = directory.clone();
        println!("new conn");
        thread::spawn(move || handle_request_helper(stream, dir));
        // match stream {
        //     Ok(mut stream) => {
        //         let request = Request::new(&stream);
        //         println!("request is: {:?}", request);
        //         thread::spawn(move || handle_request(request, &mut stream, dir));
        //     }
        //     Err(e) => println!("Error {}", e),
        // }
    }
}
