use axum::extract::{Multipart, Path};
use axum::response::{IntoResponse, Redirect, Response};
use std::{fs, io};
use std::io::Write;
use axum::body::{Body, Bytes};
use axum::BoxError;
use axum::http::{header, StatusCode};
use futures::Stream;
use futures_util::TryFutureExt;
use rand::random;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;
use crate::backup::sync_files;
use futures_util::TryStreamExt;

pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let mut name_field = None;
    let mut original_file_name = None;
    let mut original_file_ending = None;
    let mut data = false;
    let random = random::<u64>();
    let tmp_file_name = format!("tmp_{}", random);
    let tmp_folder ="./smbackup_tmp";
    if !fs::metadata(tmp_folder).is_ok() {
        fs::create_dir(tmp_folder).unwrap();
    }

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
            let path = format!("{}/{}", tmp_folder,tmp_file_name);
            stream_to_file(&path, field).await.unwrap();
            println!("Data written");;
            data = true;
        }
    }

    return if data {
        let mut file_name;
        if name_field.is_none() {
            file_name = format!("{}", &original_file_name.unwrap());
        } else{
            file_name = name_field.unwrap();
            if original_file_ending.is_some() {
                file_name.push_str(&format!(".{}", original_file_ending.unwrap()));
            }
        }
        println!("Renaming file to {}", file_name);
        fs::rename(format!("{}/{}", tmp_folder,tmp_file_name), format!("{}/{}", crate::fs_utils::get_main_loc(), file_name)).unwrap();
        sync_files().expect("TODO: panic message");
        Redirect::to("/").into_response()
    } else {
        Response::builder().status(400).body("Bad Request".into()).unwrap()
    };
}

pub async fn show(Path(file_name): Path<String>, download: bool) -> Response {
    let main_loc = crate::fs_utils::get_main_loc();
    let file_path = format!("{}/{}", main_loc, file_name);
    if let Ok(data) = fs::read(file_path) {
        let body = Body::from(data);
        let guess = mime_guess::from_path(file_name.clone()).first_or_octet_stream();
        let mut headers = [
            (header::CONTENT_TYPE, guess.to_string()),
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

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<BoxError>,
{
    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err.into()));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        println!("Streaming to file {}", &path);
        let path = std::path::Path::new(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}
