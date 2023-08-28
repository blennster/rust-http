use std::{collections::HashMap, dbg, io::prelude::*, str::FromStr};

type RouteMap = HashMap<String, fn(HttpRequest) -> HttpResponse>;

#[derive(Default)]
pub struct HttpServer {
    routes: RouteMap,
}

pub enum HttpMethod {
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

#[derive(Default)]
pub struct HttpResponse {
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
    pub content_type: String,
}
pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub method: HttpMethod,
    pub uri: String,
}

fn status_code_message(status_code: u16) -> String {
    match status_code {
        200 => "OK".to_string(),
        400 => "Bad Request".to_string(),
        404 => "Not Found".to_string(),
        500 => "Internal Server Error".to_string(),
        _ => panic!("invalid status code"),
    }
}

impl FromStr for HttpRequest {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splt = s.split("\r\n");
        let s = splt.next().unwrap();

        let req: Result<Self, Self::Err> = {
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
                headers: HashMap::new(),
            })
        };
        let mut req = req?;

        let s = splt.next().unwrap();
        {
            let splt = s.split('\n');
            for s in splt {
                if s.is_empty() {
                    continue;
                }
                let mut split = s.split(':');
                let key = match split.next() {
                    Some(k) => k,
                    None => continue,
                };
                let value = match split.next() {
                    Some(v) => v,
                    None => continue,
                };
                req.headers.insert(key.to_string(), value.to_string());
            }
        };

        Ok(req)
    }
}

impl HttpServer {
    fn verify_route(&self, route: &str) -> bool {
        let starts_with_slash = route.starts_with('/');
        let no_whitespaces = !route.contains(' ');
        let not_duplicate = !self.routes.keys().any(|k| k == route);

        starts_with_slash && no_whitespaces && not_duplicate
    }

    pub fn register_route(
        &mut self,
        route: &str,
        handler: fn(HttpRequest) -> HttpResponse,
    ) -> &mut Self {
        if !self.verify_route(route) {
            panic!("invalid route: {}", route);
        }
        self.routes.insert(route.to_owned(), handler);

        self
    }

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

        let mut handlers = self
            .routes
            .keys()
            .filter(|k| glob(k, &req.uri))
            .collect::<Vec<&String>>();
        handlers.sort();

        let handler = self.routes.get(*handlers.last().unwrap());
        let mut response = match handler {
            Some(handler) => match std::panic::catch_unwind(|| handler(req)) {
                Ok(res) => res,
                Err(error) => {
                    dbg!(error);
                    HttpResponse {
                        status_code: 500,
                        ..Default::default()
                    }
                }
            },
            None => HttpResponse {
                status_code: 404,
                ..Default::default()
            },
        };

        HttpServer::write_response(&mut stream, &mut response)
    }

    fn write_response(
        stream: &mut std::net::TcpStream,
        response: &mut HttpResponse,
    ) -> std::io::Result<()> {
        let http_status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            response.status_code,
            status_code_message(response.status_code)
        );

        response
            .headers
            .insert("Server".to_owned(), "rust-http/0.1".to_owned());
        response
            .headers
            .insert("Content-Length".to_owned(), response.body.len().to_string());
        response
            .headers
            .insert("Content-Type".to_owned(), response.content_type.clone());

        let mut headers = response
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect::<Vec<String>>();
        headers.sort();

        let response = [
            http_status_line,
            headers.join(""),
            "\r\n".to_string(),
            std::str::from_utf8(response.body.as_slice())
                .unwrap()
                .to_string(),
        ];

        dbg!(&response);

        stream.write_all(response.join("").as_bytes())?;
        stream.flush()
    }
}

fn glob(pattern: &str, s: &str) -> bool {
    let sc: Vec<char> = s.chars().collect();
    let n = sc.len();
    let pc: Vec<char> = pattern.chars().collect();
    let m = pc.len();
    let mut i = 0;
    let mut j = 0;
    let mut start_index = None;
    let mut ma = 0;

    while i < n {
        // If the current characters match or the
        // pattern has a '?', move to the next
        // characters in both pattern and text.
        if j < m && (pc[j] == '?' || pc[j] == sc[i]) {
            i += 1;
            j += 1;
        }
        // If the pattern has a '*' character, mark the
        // current position in the pattern and the text
        // as a proper match.
        else if j < m && pc[j] == '*' {
            start_index = Some(j);
            ma = i;
            j += 1
        }
        // If we have not found any match and no '*' character,
        // backtrack to the last '*' character position
        // and try for a different match.
        else if let Some(start_index) = start_index {
            j = start_index + 1;
            ma += 1;
            i = ma;
        }
        // If none of the above cases comply, the pattern
        // does not match.
        else {
            return false;
        }
    }

    // Consume any remaining '*' characters in the given
    // pattern.
    while j < m && pc[j] == '*' {
        j += 1
    }

    // If we have reached the end of both the pattern
    // and the text, the pattern matches the text.
    j == m
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_glob() {
        assert!(crate::glob("a*", "asdfas"));
        assert!(!crate::glob("a*", "bssasa"));
        assert!(crate::glob("a*a", "abba"));
    }
}
