use actix_web::{web, middleware, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_files::NamedFile;

use askama::Template;

mod api;

// Creates response containing file data
pub fn return_file(req: &HttpRequest, path: String) -> impl Responder {
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
    HttpResponse::Ok().body("")
}

// Wrapper on return_file to get dependency
async fn retrieve_dependencies(req: HttpRequest, name: web::Path<String>) -> impl Responder {
    return_file(&req, format!("web/deps/{}", name))
}

#[derive(Template)]
#[template(path = "explore.html")]
struct ExploreTemplate<'a> {
    id: &'a str
}

// Creates response that lets the user explore the association graph for a given generated model
async fn explore(id: web::Path<String>) -> impl Responder {
    let render_html = ExploreTemplate {
        id: id.as_str()
    }.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(render_html)
}

pub async fn launch_web_server(address: &str) -> Result<(), String> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(index))
            .route("/deps/{file_name}", web::get().to(retrieve_dependencies))

            .route("/explore/{id}", web::get().to(explore))
            .route("/explore/{id}/graph", web::get().to(api::get_association_graph))
            .route("/explore/{id}/corpus", web::get().to(api::get_corpus))
            .route("/explore/{id}/stats", web::get().to(api::get_stats))
            .route("/explore/{id}/interesting_words/{num_best}", web::get().to(api::get_interesting_words))
            .route("/explore/{id}/similar_docs/{doc_id}/{num_best}", web::get().to(api::get_similar_docs))
    })
    .bind(address).map_err(|_| "Failed to bind")?
    .run()
    .await.map_err(|_| "Failed to launch server")?;

    Ok(())
}