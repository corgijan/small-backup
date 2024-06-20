use axum::extract::{Multipart, Path};
use axum::response::{IntoResponse, Redirect, Response};
use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;
use axum::body::{Body, Bytes};
use axum::{BoxError, Form};
use axum::http::{header, StatusCode};
use futures::Stream;
use futures_util::TryFutureExt;
use rand::random;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_util::io::StreamReader;
use crate::backup::sync_files;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use crate::main;



pub async fn create_folder(Path(folder_path): Path<String>) -> impl IntoResponse {
    dbg!("CREATE FOLDER");
    dbg!(&folder_path);
    if folder_path.is_empty() || folder_path.contains("..") ||  folder_path.contains("\\") || folder_path.contains(":") {
        return Response::builder()
            .status(400)
            .body("Bad Request".to_string())
            .unwrap();
    }
    let main_loc = crate::fs_utils::get_main_loc();
    let folder_path = dbg!(format!("{}/{}", main_loc, folder_path));
    if !PathBuf::from(&folder_path).exists() {
        fs::create_dir(&folder_path).unwrap();
    }
    sync_files().unwrap();
    Response::builder()
        .status(200)
        .body("Folder created".to_string())
        .unwrap()
}

pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let mut name_field = None;
    let mut original_file_name = None;
    let mut original_file_ending = None;
    let mut data = false;
    let random = random::<u64>();
    let tmp_folder = "./smbackup_tmp";
    let tmp_file_name = format!("tmp_{}",random);
    let mut chunk_index: Option<u32> = None;
    let mut total_chunks: Option<u32> = None;
    let mut proper_file_name = None;
    let mut path = None;

    if !PathBuf::from(tmp_folder).exists() {
        fs::create_dir(tmp_folder).unwrap();
    }
    let mut file = File::create(format!("{}/{}", tmp_folder, tmp_file_name)).await.unwrap();
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("name") => {
                let bind = field.text().await.ok();
                if let Some(bind) = bind {
                    if bind.is_empty() {
                        name_field = None;
                    } else {
                        name_field = Some(bind);
                    }
                }
            }
            Some("file") => {
                let mut file_data = field.bytes().await.unwrap();

                if let Some(idx) = chunk_index {
                    let chunk_path = format!("{}/{}_{}", tmp_folder, tmp_file_name, idx);
                    stream_to_file(&chunk_path,file_data.clone()).await.unwrap();
                    data = true;
                }else {
                    let chunk_path = format!("{}/{}_tmp", tmp_folder, tmp_file_name);
                    stream_to_file(&chunk_path,file_data.clone()).await.unwrap();
                    data = true;
                }
            }
            Some("chunk") => {
                let bind = field.text().await.ok();
                let rewrite_file = data;
                chunk_index = bind.and_then(|s| s.parse::<u32>().ok());
                if rewrite_file {
                    fs::rename(format!("{}/{}_tmp", tmp_folder, tmp_file_name), format!("{}/{}_{}", tmp_folder, tmp_file_name, chunk_index.unwrap())).unwrap();
                }

            }
            Some("originalFilename") => {
                let bind = field.text().await.ok();
                original_file_name = bind.map(|s| s.to_string());
                proper_file_name = original_file_name.clone();
                if let Some(ref name) = original_file_name {
                    if name.contains('.') {
                        original_file_ending = name.split('.').last().map(|s| s.to_string());
                    }
                }
            }
            Some("uploadPath") => {
                 path = dbg!(field.text().await.ok());
            }
            Some("totalChunks") => {
                let bind = field.text().await.ok();
                total_chunks = bind.and_then(|s| s.parse::<u32>().ok());
            }
            _ => {}
        }
    }
    dbg!(&path);

    let proper_file_name = original_file_name.clone().unwrap();
    let cclone= chunk_index.clone().unwrap();
    fs::rename(format!("{}/{}_{}", tmp_folder, tmp_file_name,cclone), format!("{}/{}_{}", tmp_folder,proper_file_name,cclone)).unwrap();
    println!("original file name: {:?}", original_file_name);

    if data && chunk_index.is_some() && total_chunks.is_some() && chunk_index == total_chunks && path.is_some() {
        let mut file_name;
        if name_field.is_some() {
            file_name = name_field.unwrap();
            if let Some(ext) = original_file_ending {
                file_name.push_str(&format!(".{}", ext));
            }
        }
        else {
            file_name =  original_file_name.unwrap();
        }

        let final_path = format!("{}{}/{}", crate::fs_utils::get_main_loc(), path.unwrap(),file_name);
        // return error if file already exists
        if PathBuf::from(&final_path).exists() {
            return Response::builder()
                .status(StatusCode::CONFLICT)
                .header("cause","File already exists")
                .body("File already exists".into())
                .unwrap();
        }

        let mut final_file = fs::File::create(&final_path).unwrap();
        println!("Final path: {}", final_path);
        let chunk_num = total_chunks.unwrap();

        for i in 0..chunk_num+1 {
            let chunk_path = format!("{}/{}_{}", tmp_folder,proper_file_name, i);
            println!("writing to final file: {}/{}", i,chunk_num);
            let mut chunk_file = fs::File::open(&chunk_path).unwrap();
            std::io::copy(&mut chunk_file, &mut final_file).unwrap();
            fs::remove_file(chunk_path).unwrap();
        }
        sync_files().expect("Failed to sync files");
        println!("Done");
        return Redirect::to("/").into_response();
    } else if data && chunk_index.is_some() && total_chunks.is_some() && total_chunks.unwrap() > chunk_index.unwrap() {
        let completeness = chunk_index.unwrap() as f32 / total_chunks.unwrap() as f32 * 100.0;
        let json_comp = "{\"completeness\": ".to_string() + &completeness.to_string() + "}";
        Response::builder()
            .status(200)
            .body(json_comp.into())
            .unwrap()
    }
    else {
        Response::builder()
            .status(400)
            .body("Bad Request".into())
            .unwrap()
    }
}
async fn stream_to_file(path: &str, mut data: Bytes) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(&*data)?;
    Ok(())
}
pub async fn show_handler(Path(file_name): Path<String>, download: bool) -> Response {
    return show(Path(file_name), download).await.unwrap_or_else(|e| {
        Response::builder()
            .status(404)
            .body(format!("Error: {}", e).into())
            .unwrap()
    });
}

pub async fn show(Path(file_name): Path<String>, download: bool) -> Result<Response,anyhow::Error> {
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
        Ok((headers, body).into_response())
    } else {
        Ok(Response::builder()
            .status(404)
            .body("File not found".into())
            .unwrap())
    }
}


