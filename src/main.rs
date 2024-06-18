use axum::{extract::{Multipart, Path, Query}, response::{Html, IntoResponse, Redirect, Response}, routing::{get, post}, Router};
use futures_util::stream::StreamExt;
use std::fs;
use axum::body::{Body, Bytes};
use axum::http::header;

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
    let mut name_field= None;
    let mut original_file_name = None;
    let mut original_file_ending= None;
    let mut data = None;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("name") {
            let bind = field.text().await.ok();
            if let Some(bind) = bind {
                if bind == "" {
                    name_field = None;
                } else {
                    name_field = Some(bind);
                }
            }

        } else if field.name() == Some("file") {
            original_file_name = Some(dbg!(field.file_name().unwrap().to_string()));
            original_file_ending = Some(original_file_name.clone().unwrap().split('.').last().unwrap().to_string());
            let bind = field.bytes().await.unwrap();
            data = Some(bind);
        }
    }

    return if name_field.is_some() && data.is_some(){
        let file_name = format!("{}.{}", &name_field.unwrap(), &original_file_ending.unwrap());
        write_file(file_name, data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else if name_field.is_none() && data.is_some() && original_file_name.is_some() && original_file_ending.is_some() && original_file_ending.is_some(){
        write_file(original_file_name.unwrap(), data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else {
        Response::builder().status(400).body("Bad Request".into()).unwrap()
    }
}

async fn upload_form() -> Html<String> {
    let paths = fs::read_dir(std::env::var("MAIN_LOCATION").expect("MAIN_LOCATION not set")).expect("Failed to read main directory");
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
            <input type='text' name='name' />
            <input type='submit' value='Upload File' />
            <br>{}
        </form> {}",
        paths, STYLE
    ))
}

async fn download(Path(file_name): Path<String>) -> Response{
    let file_path = format!("{}/{}",std::env::var("MAIN_LOCATION").expect("MAIN_LOCATION not set"), file_name);
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

async fn write_file(file_name: String, data: Bytes) -> Result<(), std::io::Error> {
    let locations = std::env::var("REPLICATION_LOCATIONS").unwrap_or(std::env::var("MAIN_LOCATION").expect("MAIN_LOCATION not set"));
    for loc in locations.split(":"){
        fs::write(format!("{}/{}",loc, file_name), data.clone()).expect(format!("Failed to write file to {}", loc).as_str());
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let app = Router::new()
        .route("/", get(upload_form))
        .route("/upload", post(upload))
        .route("/serve/:file_name", get(download));
    println!("Little_Share :: Listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
