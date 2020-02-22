use actix_web::{web, HttpResponse, Responder, HttpRequest};

use std::path::PathBuf;
use std::fs;

use std::process::Command;

use crate::web_server::return_file;

// Constants that point to python analysis scripts
const SCRIPTS_FOLDER: &str = "../scripts/";
const PREPROCESS_SCRIPT: &str = "preprocess.py";
const ASSOCIATE_SCRIPT: &str = "associate_words.py";
const SIMILARITY_SCRIPT: &str = "similarity.py";

fn get_models_dir(id: &String) -> PathBuf {
    fs::canonicalize(format!("web/data/{}/models", id)).unwrap()
}

// API endpoint to get the association graph for a given generated model
pub async fn get_association_graph(req: HttpRequest, info: web::Path<String>) -> impl Responder {
    let id = &info;
    return_file(&req, format!("web/data/{}/association_graph.json", id))
}

// API endpoint to get the corpus for a given generated model
pub async fn get_corpus(req: HttpRequest, info: web::Path<String>) -> impl Responder {
    let id = &info;
    return_file(&req, format!("web/data/{}/models/corpus_data.json", id))
}

// API endpoint to stats of ads by returning json file generated during ad collection
pub async fn get_stats(req: HttpRequest, info: web::Path<String>) -> impl Responder {
    let id = &info;
    return_file(&req, format!("web/data/{}/ad_data.json", id))
}

// API endpoint to find interesting words for a given model by executing python script
fn interesting_words(id: &String, num_best: usize) -> impl Responder {
    // Find the directory with generated models ... get absolute path for python script
    let models_dir = get_models_dir(id);
    // Execute the script and the script will output json response
    let command = Command::new("python3")
        .current_dir(SCRIPTS_FOLDER)
        .args(&[
            SIMILARITY_SCRIPT,
            "interesting_words",
            format!("{}", models_dir.as_path().display()).as_str(),
            format!("--num_best={}", num_best).as_str()])
        .output();
    match command {
        Ok(output) => {
            if !output.status.success() {
                return core::result::Result::Err(actix_web::Error::from(HttpResponse::InternalServerError().body("Script failed")));
            }
            let stdout = String::from_utf8(output.stdout);
            match stdout {
                Ok(res) => {
                    core::result::Result::Ok(HttpResponse::Ok().content_type("application/json").body(res))
                },
                Err(_) => {
                    core::result::Result::Err(actix_web::Error::from(HttpResponse::InternalServerError().body("Script failed")))
                },
            }
        },
        Err(_) => {
            core::result::Result::Err(actix_web::Error::from(HttpResponse::InternalServerError().body("Command failed")))
        },
    }
}

pub async fn get_interesting_words(info: web::Path<(String, usize)>) -> impl Responder {
    let id = &info.0;
    let num_best = info.1;
    interesting_words(id, num_best)
}

fn similar_docs(id: &String, doc_id: usize, num_best: usize) -> impl Responder {
    // Find the directory with generated models ... get absolute path for python script
    let models_dir = get_models_dir(id);
    // Execute the script and the script will output json response
    let command = Command::new("python3")
        .current_dir(SCRIPTS_FOLDER)
        .args(&[
            SIMILARITY_SCRIPT,
            "similar_docs",
            format!("{}", models_dir.as_path().display()).as_str(),
            format!("{}", doc_id).as_str(),
            format!("--num_best={}", num_best).as_str()])
        .output();
    match command {
        Ok(output) => {
            if !output.status.success() {
                return core::result::Result::Err(actix_web::Error::from(HttpResponse::InternalServerError().body("Script failed")));
            }
            let stdout = String::from_utf8(output.stdout);
            match stdout {
                Ok(res) => {
                    core::result::Result::Ok(HttpResponse::Ok().content_type("application/json").body(res))
                },
                Err(_) => {
                    core::result::Result::Err(actix_web::Error::from(HttpResponse::InternalServerError().body("Script failed")))
                },
            }
        },
        Err(_) => {
            core::result::Result::Err(actix_web::Error::from(HttpResponse::InternalServerError().body("Command failed")))
        },
    }
}

pub async fn get_similar_docs(info: web::Path<(String, usize, usize)>) -> impl Responder {
    let id = &info.0;
    let doc_id = info.1;
    let num_best = info.2;
    similar_docs(id, doc_id, num_best)
}