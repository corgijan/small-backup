use axum::{extract::Path, response::IntoResponse, Router, routing::{get, post}};
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderMap;
use axum::response::{Html, Redirect, Response};
use futures_util::stream::StreamExt;
use crate::file_handlers::create_folder;

mod index_page;
mod file_handlers;
mod backup;
mod fs_utils;

#[tokio::main]
async fn main()-> Result<(), anyhow::Error>{
    dotenv::dotenv().ok();
    let main_loc = fs_utils::get_main_loc();
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let max_body = std::env::var("MAX_BODY").unwrap_or("20971520".to_string()).parse().unwrap();

    let app = Router::new()
        .route("/", get(index_page::upload_form_main))
        .route("/create_folder/*key", post(create_folder))
        .route("/*key", get(index_page::upload_form))
        .route("/upload", post(file_handlers::upload))
        .route("/upload/*key", post(file_handlers::upload))
        .route("/upload/*key", get(|Path(path): Path<String>|async move {Redirect::to(&*("/".to_owned() + &*path)).into_response()}))
        .route("/upload", get(||async {Redirect::to("/").into_response()}))
        .route("/download/*key", get(|file_name: Path<String>| async { file_handlers::show_handler(file_name, true).await }))
        .layer(DefaultBodyLimit::max( max_body));

    backup::generate_all_folders()?;
    println!("Syncing files");
    backup::sync_files()?;
    println!("Syncing done");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Little_Share :: Listening on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handler_wildcard(Path(path): Path<String>) -> Html<String> {
    Html(format!("{:?}\n",path))
}
