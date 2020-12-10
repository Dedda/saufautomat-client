use askama::Template;
use rouille::Response;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    nbevs: usize,
    nfiles: usize,
}

pub fn index(nbevs: usize, nfiles: usize) -> Response {
    Response::html(Index{ nbevs, nfiles }.render().unwrap())
}

#[derive(Template)]
#[template(path = "nbevs.html")]
struct NBevs {
    n: usize,
}

pub fn nbevs(n: usize) -> Response {
    let nbevs = NBevs { n };
    Response::html(nbevs.render().unwrap())
}