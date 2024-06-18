use axum::extract::{Multipart, Path};
use axum::response::{IntoResponse, Redirect, Response};
use std::fs;
use axum::body::Body;
use axum::http::header;

pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
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
        crate::backup::write_file(file_name, data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else if name_field.is_none() && data.is_some() && original_file_name.is_some() && original_file_ending.is_some() && original_file_ending.is_some() {
        crate::backup::write_file(original_file_name.unwrap(), data.unwrap()).await.unwrap();
        Redirect::to("/").into_response()
    } else if name_field.is_none() && data.is_some() && original_file_name.is_some() && original_file_ending.is_none() && original_file_ending.is_some() {
        crate::backup::write_file(original_file_name.unwrap(), data.unwrap()).await.unwrap();
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
