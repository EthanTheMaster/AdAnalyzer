use actix_web::{web, HttpResponse, Responder, HttpRequest};

use std::path::PathBuf;
use std::fs;

use serde_json;

use std::process::Command;

use crate::web_server::return_file;

// Constants that point to python analysis scripts
const SCRIPTS_FOLDER: &str = "../scripts/";
const PREPROCESS_SCRIPT: &str = "preprocess.py";
const ASSOCIATE_SCRIPT: &str = "associate_words.py";
const SIMILARITY_SCRIPT: &str = "similarity.py";

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

// API endpoint to find interesting words for a given model by executing python script
fn interesting_words(id: &String, num_best: usize) -> impl Responder {
    // Find the directory with generate models ... get absolute path for python script
    let models_dir = fs::canonicalize(format!("web/data/{}/models", id)).unwrap();
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

pub async fn get_interesting_words(req: HttpRequest, info: web::Path<(String, usize)>) -> impl Responder {
    let id = &info.0;
    let num_best = info.1;
    interesting_words(id, num_best)
}