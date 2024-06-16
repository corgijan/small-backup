use axum::{extract::{Multipart, Path, Query}, response::{Html, IntoResponse, Redirect, Response}, routing::{get, post}, Router};
use futures_util::stream::StreamExt;
use std::fs;
use axum::body::Body;
use axum::http::header;

static UPLOAD_DIR: &str = "./data";
static PW_CHECK: bool = true;
static PASSWORD: &str = "password";
static STYLE: &str = "<style>\
body { font-family: sans-serif; }\
input[type='file'] { margin-bottom: 1em; }\
.item { \
    margin-top: 1em; \
    background-color: lightgreen; \
    width: 400px; \
    height: 100px; \
    margin: 10px; \
    display: flex; \
    justify-content: center; \
    align-items: center; \
    text-align: center; \
    padding: 10px; \
    border-radius: 8px; \
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); \
}\
</style>";

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        fs::write(format!("{}/{}", UPLOAD_DIR, &name), data).unwrap();
    }
    Redirect::to("/").into_response()
}

async fn upload_form() -> Html<String> {
    let paths = fs::read_dir(UPLOAD_DIR).unwrap();
    let paths = paths
        .map(|entry| {
            entry.map(|e| {
                let file_name = e.file_name().to_str().unwrap().to_string();
                format!(
                    "<div class='item'><a href='/serve/{}'>{}</a></div>",
                    file_name, file_name,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("<br>");
    Html(format!(
        "<form action='/upload' method='post' enctype='multipart/form-data'>
            <input type='file' name='file' />
            <input type='submit' />
            <br>{}
        </form> {}",
        paths, STYLE
    ))
}

async fn download(Path(file_name): Path<String>) -> Response{
    let file_path = format!("{}/{}", UPLOAD_DIR, file_name);
    if let Ok(data) = fs::read(file_path) {
        let body = Body::from(data);
        let headers = [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{:?}\"", file_name),
            ),
        ];
        (headers, body).into_response()
    } else {
        Response::builder()
            .status(404)
            .body("File not found".into())
            .unwrap()
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let app = Router::new()
        .route("/", get(upload_form))
        .route("/upload", post(upload))
        .route("/serve/:file_name", get(download));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
