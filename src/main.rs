use std::io;

use actix_web::{get, web, App, HttpResponse, HttpServer};
use awc::Client;

use rand::Rng;
use std::cmp::Ordering;

#[get("/location/{latitude}/{longitude}")]
async fn get_location(path: web::Path<(f64, f64)>) -> HttpResponse {
    let (lat, long) = path.into_inner();
    let response = format!("Latitude: {}, Longitude: {}", lat, long);
    HttpResponse::Ok().body(response)
}

async fn query_glocation() {
    // let mut client awc::Client::default();
    // let req = client.get("http://www.rust-lang.org");
}

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
// HttpServer::new(|| App::new().service(get_location))
// .bind("0.0.0.0:8080")?
// .run()
// .await
// }

fn mut_test(buf: &mut String) -> u32 {
    buf.push('F');
    3
}

fn main() {
    println!("Guess the number!");
    println!("Put your guess here: ");

    let mut input: String = String::new();
    let secret_number: i32 = 50;
    io::stdin().read_line(&mut input).expect("Bad");
    println!("You guessed {}", input);

    let input: i32 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => -1,
    };

    if input == -1 {
        println!("HI!");
        return;
    }

    match input.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }
    // mut_test(&mut input);
}
