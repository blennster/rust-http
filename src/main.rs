use rust_http::*;

fn pong(_: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        body: r#"
            <!DOCTYPE html>
            <html>
                <head>
                <title>pong</title>
                </head>
                <body>pong</body>
            </html>"#
            .as_bytes()
            .to_vec(),
        content_type: "text/html".to_string(),
        ..Default::default()
    }
}

fn handle(req: HttpRequest) -> HttpResponse {
    let mut cwd = std::env::current_dir().unwrap();
    cwd.push(req.uri.strip_prefix('/').unwrap());

    if !cwd.exists() {
        return HttpResponse {
            status_code: 404,
            ..Default::default()
        };
    }

    match cwd.is_dir() {
        true => {
            let dirs = std::fs::read_dir(cwd)
                .unwrap()
                .map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect::<Vec<String>>()
                .join("\n")
                .as_bytes()
                .to_vec();

            HttpResponse {
                status_code: 200,
                body: dirs,
                ..Default::default()
            }
        }
        false => {
            let content = std::fs::read_to_string(cwd).unwrap().as_bytes().to_vec();
            HttpResponse {
                status_code: 200,
                body: content,
                ..Default::default()
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let port = a[1].parse::<u16>().unwrap();

    let mut server = HttpServer::default();
    server
        .register_route("/ping", pong)
        .register_route("/*", handle);

    server.listen(port)?;

    println!("Hello, world!");
    Ok(())
}
