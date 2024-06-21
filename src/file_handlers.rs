use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;

use axum::body::{Body, Bytes};
use axum::extract::{Multipart, Path};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;
use rand::random;
use tokio::fs::File;

use crate::backup::sync_files;

pub async fn create_folder_wrap(Path(folder_path): Path<String>) -> Response {
    let error_resp = Response::builder()
        .status(404)
        .body("Error".to_string().into())
        .unwrap();
    let resp = create_folder(Path(folder_path)).await;
    resp.unwrap_or_else(|e| error_resp )
}

pub async fn create_folder(Path(folder_path): Path<String>) -> Result<Response, anyhow::Error> {
    dbg!("CREATE FOLDER");
    dbg!(&folder_path);
    if folder_path.replace("/", "").chars().any(|c| !c.is_alphanumeric()) {
        return Err(anyhow::anyhow!("Invalid folder name"));
    }
    let main_loc = crate::fs_utils::get_main_loc();
    let folder_path = dbg!(format!("{}/{}", main_loc, folder_path));
    if !PathBuf::from(&folder_path).exists() {
        fs::create_dir(&folder_path)?;
    }
    sync_files()?;
    Ok(Redirect::to("/").into_response())
}

pub async fn upload_wrap(mut multipart: Multipart) -> impl IntoResponse {
    upload_file(multipart).await.unwrap_or_else(|e| {
        Response::builder()
            .status(404)
            .body(format!("Error: {}", e).into())
            .unwrap()
    })
}

pub async fn upload_file(mut multipart: Multipart) -> Result<Response, anyhow::Error> {
    let mut name_field = None;
    let mut original_file_name = None;
    let mut original_file_ending = None;
    let mut data = false;
    let random = random::<u64>();
    let tmp_folder = "./smbackup_tmp";
    let tmp_file_name = format!("tmp_{}", random);
    let mut chunk_index: Option<u32> = None;
    let mut total_chunks: Option<u32> = None;
    let mut proper_file_name = None;
    let mut path = None;

    if !PathBuf::from(tmp_folder).exists() {
        fs::create_dir(tmp_folder)?;
    }
    //check if filename  contains unsafe system characters


    let mut file = File::create(format!("{}/{}", tmp_folder, tmp_file_name)).await?;
    while let Some(mut field) = multipart.next_field().await? {
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
                let mut file_data = field.bytes().await?;

                if let Some(idx) = chunk_index {
                    let chunk_path = format!("{}/{}_{}", tmp_folder, tmp_file_name, idx);
                    stream_to_file(&chunk_path, file_data.clone()).await?;
                    data = true;
                } else {
                    let chunk_path = format!("{}/{}_tmp", tmp_folder, tmp_file_name);
                    stream_to_file(&chunk_path, file_data.clone()).await?;
                    data = true;
                }
            }
            Some("chunk") => {
                let bind = field.text().await.ok();
                let rewrite_file = data;
                chunk_index = bind.and_then(|s| s.parse::<u32>().ok());
                if rewrite_file {
                    fs::rename(format!("{}/{}_tmp", tmp_folder, tmp_file_name), format!("{}/{}_{}", tmp_folder, tmp_file_name, chunk_index.unwrap()))?;
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

    let proper_file_name = original_file_name.clone().ok_or(anyhow::anyhow!("Original file name not found"))?;
    let cclone = chunk_index.clone().ok_or(anyhow::anyhow!("Chunk index not found"))?;
    fs::rename(format!("{}/{}_{}", tmp_folder, tmp_file_name, cclone), format!("{}/{}_{}", tmp_folder, proper_file_name, cclone))?;
    println!("original file name: {:?}", original_file_name);

    if data && chunk_index.is_some() && total_chunks.is_some() && chunk_index == total_chunks && path.is_some() {
        let mut file_name;
        if name_field.is_some() && name_field != original_file_name{
            file_name = name_field.ok_or(anyhow::anyhow!("Name field not found"))?;
            if let Some(ext) = original_file_ending {
                file_name.push_str(&format!(".{}", ext));
            }
        } else {
            file_name = original_file_name.ok_or(anyhow::anyhow!("Original file name not found"))?;
        }


        let final_path = format!("{}{}/{}", crate::fs_utils::get_main_loc(), path.unwrap(), file_name);
        // return error if file already exists
        if PathBuf::from(&final_path).exists() {
            return Ok(Response::builder()
                .status(StatusCode::CONFLICT)
                .header("cause", "File already exists")
                .body("File already exists".into())?);
        }

        let mut final_file = fs::File::create(&final_path)?;
        println!("Final path: {}", final_path);
        let chunk_num = total_chunks.ok_or(anyhow::anyhow!("Total chunks not found"))?;

        for i in 0..chunk_num + 1 {
            let chunk_path = format!("{}/{}_{}", tmp_folder, proper_file_name, i);
            println!("writing to final file: {}/{}", i, chunk_num);
            let mut chunk_file = fs::File::open(&chunk_path)?;
            std::io::copy(&mut chunk_file, &mut final_file)?;
            fs::remove_file(chunk_path)?;
        }
        sync_files().expect("Failed to sync files");
        println!("Done");
        return Ok(Redirect::to("/").into_response());
    } else if data && chunk_index.is_some() && total_chunks.is_some() && total_chunks.unwrap() > chunk_index.unwrap() {
        let completeness = chunk_index.unwrap() as f32 / total_chunks.unwrap() as f32 * 100.0;
        let json_comp = "{\"completeness\": ".to_string() + &completeness.to_string() + "}";
        Ok(Response::builder()
            .status(200)
            .body(json_comp.into())
            ?)
    } else {
        Ok(Response::builder()
            .status(400)
            .body("Bad Request".into())
            ?)
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

pub async fn show(Path(file_name): Path<String>, download: bool) -> Result<Response, anyhow::Error> {
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
            .body("File not found".into())?)
    }
}


