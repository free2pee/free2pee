use actix_web::{get, web, App, HttpResponse, HttpServer};

#[get("/location/{latitude}/{longitude}")]
async fn get_location(
    path: web::Path<(f64, f64)>,
) -> HttpResponse {
    let (lat, long) = path.into_inner();
    let response = format!("Latitude: {}, Longitude: {}", lat, long);
    HttpResponse::Ok().body(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_location))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
