use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(_: HttpRequest) -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().route("/hello", web::get().to(greet)))
        .bind("127.0.0.1:3000")?
        .run()
        .await
}
