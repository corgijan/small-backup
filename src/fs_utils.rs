use axum::Extension;
use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct File {
    name: String,
    size: String,
    extension: String
}

pub fn read_files(location: impl ToString)-> Result<Vec<File>,anyhow::Error>{
    let loc = location.to_string();
    let paths = std::fs::read_dir(loc)?;
    let paths = paths
        .map(|entry| {
            entry.map(|e| {
                let file_name = e.file_name().to_str().unwrap().to_string();
                let file_size = e.metadata().unwrap().len();
                let file_size = if file_size > 1024 * 1024 * 1024 {
                    format!("{} GB", file_size / (1024 * 1024 * 1024))
                }
                else if file_size > 1024 * 1024 {
                    format!("{} MB", file_size / (1024 * 1024))
                }
                else if file_size > 1024 {
                    format!("{} KB", file_size / 1024)
                } else {
                    format!("{} B", file_size)
                };
                if file_name.contains(".") {
                    let extension = file_name.split('.').last().unwrap().to_string();
                    File {
                        name: file_name,
                        size: file_size,
                        extension
                    }
                } else {
                    File {
                        name: file_name,
                        size: file_size,
                        extension: "".to_string()
                    }
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    Ok(paths)
}


pub fn get_main_loc() -> String {
    let bind = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATION not set, please set in ENV or .env file");
    let main_loc = bind.split(":").collect::<Vec<&str>>()[0];
    main_loc.to_string()
}
