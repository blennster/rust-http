fn main() -> std::io::Result<()> {
    let server = rust_http::HttpServer {};
    server.listen(8080)?;

    println!("Hello, world!");
    Ok(())
}
