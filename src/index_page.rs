use axum::response::{Html, IntoResponse, Response};
use std::fs;
use axum::extract::Path;
use minijinja::{context, Environment};
use crate::fs_utils::read_files;


pub async fn upload_form(Path(path): Path<String>) -> impl IntoResponse {
    let resp = file_overview_handler(path).await;
    let error_resp = Response::builder()
        .status(404)
        .body("Error".to_string().into())
        .unwrap();
    resp.unwrap_or_else(|e| error_resp)
}
pub async fn upload_form_main() -> impl IntoResponse {
    let resp = file_overview_handler("/".to_string()).await;
    let error_resp = Response::builder()
        .status(404)
        .body("Error".to_string().into())
        .unwrap();
    resp.unwrap_or_else(|e| error_resp )
}

pub async fn file_overview_handler(path: String) -> Result<Response,anyhow::Error> {
    let path = "/".to_string() + &*path;
    let breadcrumb = path.split("/");
    let mut bread_list = Vec::new();
    let mut last_agg = "".to_string();
    for b in breadcrumb {
        last_agg = last_agg + &*b + "/";
        bread_list.push(vec![b.to_string(), last_agg.to_string()]);
    }
    let main_loc = crate::fs_utils::get_main_loc();
    let loc = dbg!(main_loc +  &*path);
    // if location does not exist, return 404
    if !fs::metadata(loc.clone()).is_ok() {
        return Err(anyhow::anyhow!("Directory {} does not exist", loc));
    }
    //if location is file return file
    if fs::metadata(loc.clone()).unwrap().is_file() {
        return crate::file_handlers::show(Path(path), false).await;
    }
    let files = read_files(loc)?;
    let mut env = Environment::new();
    env.add_template("file_overview", include_str!("../templates/file_overview.html"))?;
    let tmpl = env.get_template("file_overview")?;
    let path = path.replace("//", "");
    return Ok(Response::builder()
        .status(200)
        .body(tmpl.render(context!(files => files, current_path => path,breadcrumbs => bread_list))?.into())
        .unwrap());

}

