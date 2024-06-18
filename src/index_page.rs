use axum::response::Html;
use std::fs;
use minijinja::{context, Environment};
use crate::fs_utils::read_files;


pub async fn upload_form() -> Html<String> {
    let resp = upload_form_handler().await;
    resp.unwrap_or_else(|e| Html(format!("Error: {}", e)))
}

pub async fn upload_form_handler() -> Result<Html<String>,anyhow::Error> {
    let main_loc = crate::fs_utils::get_main_loc();
    let files = read_files(main_loc)?;
    let mut env = Environment::new();
    env.add_template("file_overview", include_str!("../templates/file_overview.html"))?;
    let tmpl = env.get_template("file_overview")?;

    Ok(Html(tmpl.render(context!(files => files))?))
}

