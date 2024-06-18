use std::fs;

use axum::{extract::{Multipart, Path}, response::{Html, IntoResponse, Redirect, Response}, Router, routing::{get, post}};
use axum::body::{Body, Bytes};
use axum::http::header;
use futures_util::stream::StreamExt;

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
    let mut name_field = None;
    let mut original_file_name = None;
    let mut original_file_ending = None;
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
            if original_file_name.clone().unwrap().contains(".") {
                let bind = original_file_name.clone();
                original_file_ending = Some(bind.clone().unwrap().split('.').last().unwrap().to_string());
            }
            let bind = field.bytes().await.unwrap();
            data = Some(bind);
        }
    }

    return if name_field.is_some() && data.is_some() {
        let mut file_name;
        if original_file_ending.is_some() {
            file_name = format!("{}.{}", &name_field.unwrap(), &original_file_ending.unwrap());
        } else {
            file_name = name_field.unwrap();
        }
        write_file(file_name, data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else if name_field.is_none() && data.is_some() && original_file_name.is_some() && original_file_ending.is_some() && original_file_ending.is_some() {
        write_file(original_file_name.unwrap(), data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else if name_field.is_none() && data.is_some() && original_file_name.is_some() && original_file_ending.is_none() && original_file_ending.is_some() {
        write_file(original_file_name.unwrap(), data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else {
        Response::builder().status(400).body("Bad Request".into()).unwrap()
    };
}

async fn upload_form() -> Html<String> {
    let main_loc = get_main_loc();
    let paths = fs::read_dir(main_loc).expect("Failed to read main directory");
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

async fn show(Path(file_name): Path<String>, download: bool) -> Response {
    let main_loc = get_main_loc();
    let file_path = format!("{}/{}", main_loc, file_name);
    if let Ok(data) = fs::read(file_path) {
        let body = Body::from(data);
        let guess = mime_guess::from_path(file_name.clone()).first_or_octet_stream();
        let mut headers = [
            (header::CONTENT_TYPE, dbg!(guess.to_string())),
        ];

        if download {
            headers = [
                (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{:?}\"", file_name.clone()))
            ]
        }
        (headers, body).into_response()
    } else {
        Response::builder()
            .status(404)
            .body("File not found".into())
            .unwrap()
    }
}

async fn write_file(file_name: String, data: Bytes) -> Result<(), std::io::Error> {
    let locations = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATION not set, please set in ENV or .env file");
    for loc in locations.split(":") {
        fs::write(format!("{}/{}", loc, file_name), data.clone()).expect(format!("Failed to write file to {}", loc).as_str());
    }
    Ok(())
}

fn get_main_loc() -> String {
    let bind = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATION not set, please set in ENV or .env file");
    let main_loc = bind.split(":").collect::<Vec<&str>>()[0];
    main_loc.to_string()
}

fn generate_all_folders() {
    let locations = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATION not set, please set in ENV or .env file");
    for loc in locations.split(":") {
        fs::create_dir_all(loc).expect(format!("Failed to create directory {}", loc).as_str());
    }
}

fn sync_files() {
    for loc2 in std::env::var("REPLICATION_LOCATIONS").unwrap().split(":") {
        for loc1 in std::env::var("REPLICATION_LOCATIONS").unwrap().split(":") {
            if loc1 != loc2 {
                println!("Syncing {} to {}", loc1, loc2);
                for file in fs::read_dir(loc1).expect("Failed to read main directory") {
                    let file = file.unwrap();
                    // if file is not in loc2 write it to loc2
                    if !fs::metadata(format!("{}/{}", loc2, file.file_name().to_str().unwrap())).is_ok() {
                        let data = fs::read(format!("{}/{}", loc1, file.file_name().to_str().unwrap())).expect("Failed to read file");
                        fs::write(format!("{}/{}", loc2, file.file_name().to_str().unwrap()), data).expect("Failed to write file");
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let main_loc = get_main_loc();
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let app = Router::new()
        .route("/", get(upload_form))
        .route("/upload", post(upload))
        .route("/serve/:file_name", get(|file_name: Path<String>| async { show(file_name, false).await }))
        .route("/download/:file_name", get(|file_name: Path<String>| async { show(file_name, true).await }));

    println!("Little_Share :: Listening on port {}", port);
    generate_all_folders();
    println!("Syncing files");
    sync_files();
    println!("Syncing done");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
