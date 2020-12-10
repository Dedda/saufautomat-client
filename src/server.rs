use std::collections::HashMap;
use crate::Beverage;
use yatl::{duration_to_human_string, Timer};

mod templates;

pub fn serve(bevs: HashMap<String, Vec<Beverage>>) {
    print!("Crunching numbers...");
    let mut timer = Timer::new();
    let _ = timer.start();
    let count: usize = bevs.iter().map(|(_, v)| v.iter().map(|b| b.count).sum::<usize>()).sum();
    let json: String = serde_json::to_string_pretty(&json!(bevs)).unwrap();
    let dur = timer.lap().unwrap();
    println!(" OK [{}]", duration_to_human_string(&dur));
    println!("The server is now available at 'localhost:3000'");

    rouille::start_server("localhost:3000", move |request| {
        router!(request,
            (GET) (/) => {
                templates::index(count, bevs.len())
            },
            (GET) (/nbevs) => {
                templates::nbevs(count)
            },
            (GET) (/dump) => {
                rouille::Response::text(&json)
            },
            _ => rouille::Response::empty_404()
        )
    });
}