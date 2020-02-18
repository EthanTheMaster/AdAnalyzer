use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_files::NamedFile;

use askama::Template;

fn return_file(req: &HttpRequest, path: String) -> impl Responder {
    match NamedFile::open(&path) {
        Ok(file) => {
            file.into_response(req)
        },
        Err(_) => {
            Err(actix_web::Error::from(HttpResponse::NotFound().body("Oops")))
        },
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("index page goes here...")
}

async fn retrieve_script(req: HttpRequest, name: web::Path<String>) -> impl Responder {
    return_file(&req, format!("web/scripts/{}", name))
}


async fn explore_data(req: HttpRequest, info: web::Path<(String, String)>) -> impl Responder {
    let id = &info.0;
    let file_name = &info.1;
    match file_name.as_str() {
        "graph" => {return_file(&req, format!("web/data/{}/association_graph.json", id))},
        "corpus" => {return_file(&req, format!("web/data/{}/models/corpus_data.json", id))},
        _ => {return_file(&req, format!("web/error"))}
    }
}

#[derive(Template)]
#[template(path = "explore.html")]
struct ExploreTemplate<'a> {
    id: &'a str
}

async fn explore(id: web::Path<String>) -> impl Responder {
    let render_html = ExploreTemplate {
        id: id.as_str()
    }.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(render_html)
}

pub async fn launch_web_server() -> Result<(), String> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/scripts/{file_name}", web::get().to(retrieve_script))
            .route("/explore/{id}/{file}", web::get().to(explore_data))
            .route("/explore/{id}", web::get().to(explore))
    })
    .bind("127.0.0.1:8080").map_err(|_| "Failed to bind")?
    .run()
    .await;

    Ok(())
}