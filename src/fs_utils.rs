use std::fmt::format;
use std::time::UNIX_EPOCH;
use axum::Extension;
use chrono::{DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct File {
    name: String,
    size: String,
    extension: String,
    created_at: String,
    relative_path: String
}

struct NaiveDateTime(i64, i32);

pub fn read_files(location: impl ToString + Clone) -> Result<Vec<File>,anyhow::Error>{
    let loc = dbg!(location.to_string());
    let replace_location_main =  get_main_loc() + "/" + "/";
    let replace_location =  get_main_loc() + "/" ;
    let relative_path = location.clone().to_string().replace(replace_location_main.as_str(), "").replace(replace_location.as_str(), "" ).replace("//", "/");
    let paths = dbg!(std::fs::read_dir(loc.clone())?);
    let paths = paths
        .map(|entry| {
            entry.map(|e| {
                let file_name = e.file_name().to_str().unwrap().to_string();
                let file_size = e.metadata().unwrap().len();
                // check if file is a folder
                if e.metadata().unwrap().is_dir() {
                    return File {
                        name: file_name,
                        size: "-".to_string(),
                        extension: "folder".to_string(),
                        created_at: "-".to_string(),
                        relative_path: relative_path.to_string()
                    }
                }
                let file_size = if file_size > 1024 * 1024 * 1024 {
                    format!("{:.1} GB", file_size as f64 / (1024 * 1024 * 1024) as f64)
                }
                else if file_size > 1024 * 1024 {
                    format!("{} MB", file_size / (1024 * 1024))
                }
                else if file_size > 1024 {
                    format!("{} KB", file_size / 1024)
                } else {
                    format!("{} B", file_size)
                };
                let creation_time;
                if std::env::var("PLATFORM").is_ok() && std::env::var("PLATFORM").unwrap() != "ARM"{
                    creation_time = DateTime::from_timestamp(e.metadata().unwrap().created().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap().to_string();
                }else {
                    creation_time = "-".to_string();
                }

                if file_name.contains(".") {
                    let extension = file_name.split('.').last().unwrap().to_string();

                    File {
                        name: file_name,
                        size: file_size,
                        extension,
                        created_at: creation_time,
                        relative_path: relative_path.to_string()
                    }
                } else {
                    File {
                        name: file_name,
                        size: file_size,
                        extension: "".to_string(),
                        created_at: creation_time,
                        relative_path: relative_path.to_string()
                    }
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    Ok(paths)
}


pub fn get_main_loc() -> String {
    let bind = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATIONS not set, please set in ENV or .env file");
    let main_loc = bind.split(":").collect::<Vec<&str>>()[0];
    main_loc.to_string()
}

/*
pub async fn read_folder_to_zip(path: impl ToString + Clone) -> Result<Bytes, anyhow::Error> {
    let path = path.into_string();
    let zip_file = tempfile::tempfile()?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut buffer = Vec::new();
    let paths = std::fs::read_dir(path.clone())?;
    for entry in paths {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            zip.start_file(file_name, options)?;
            let mut file = std::fs::File::open(path)?;
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    let mut zip_file = std::fs::File::open(zip_file)?;
    let mut zip_data = Vec::new();
    zip_file.read_to_end(&mut zip_data)?;
    Ok(Bytes::from(zip_data))

}
*/