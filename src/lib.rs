use std::{io::prelude::*, println, str::FromStr};

pub struct HttpServer {}

enum HttpMethod {
    Get,
    Post,
}

impl FromStr for HttpMethod {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Not a http method",
            )),
        }
    }
}

struct HttpResponse {}
struct HttpRequest {
    method: HttpMethod,
    uri: String,
}

fn parse_request_headers(s: &str) -> std::result::Result<HttpRequest, std::io::Error> {
    let whitespaces = s.chars().filter(|c| c.is_whitespace()).count();
    if whitespaces + 1 < 3 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Malformed http request",
        ));
    }

    let mut split = s.split_whitespace();
    let method = split.next().unwrap();
    let uri = split.next().unwrap();
    let http = split.next().unwrap();
    if http != "HTTP/1.1" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not a HTTP/1.1 request",
        ));
    }

    Ok(HttpRequest {
        method: HttpMethod::from_str(method)?,
        uri: uri.to_string(),
    })
}

impl FromStr for HttpRequest {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splt = s.split("\r\n");
        parse_request_headers(splt.next().unwrap())
    }
}

impl HttpServer {
    pub fn listen(&self, port: u16) -> std::io::Result<()> {
        println!("Listening on port {}", port);
        let listener = std::net::TcpListener::bind(("127.0.0.1", port))?;

        for stream in listener.incoming() {
            let stream = stream?;
            if let Err(msg) = self.handle(stream) {
                println!("Error: {}", msg)
            }
        }

        Ok(())
    }

    fn handle(&self, mut stream: std::net::TcpStream) -> std::io::Result<()> {
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf)?;
        let s = std::str::from_utf8(buf.as_slice()).unwrap();
        let req = HttpRequest::from_str(s)?;

        stream.write_all(
            format!("HTTP/1.1 200 OK\nServer: rust-http/0.1\r\n\r\n{}", req.uri).as_bytes(),
        )?;
        Ok(())
    }
}
