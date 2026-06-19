use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct InsertRequest {
    key: String,
    value: f64,
    timestamp: u64,
}

async fn insert(
    store: web::Data<Arc<app::Store>>,
    req: web::Json<InsertRequest>,
) -> impl Responder {
    store.insert(&req.key, req.value, req.timestamp);
    HttpResponse::Ok().json(serde_json::json!({ "status": "inserted", "total": store.count() }))
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = Arc::new(app::Store::new());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(store.clone()))
            .route("/api/v1/insert", web::post().to(insert))
            .route("/health", web::get().to(health))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
