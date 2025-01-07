```rust
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, HttpResponse, Responder};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TimeResponse {
    current_time: String,
}

async fn get_current_time() -> impl Responder {
    let time_response = TimeResponse {
        current_time: Utc::now().to_rfc3339(),
    };
    HttpResponse::Ok().json(time_response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600)
            )
            .route("/time", web::get().to(get_current_time))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```