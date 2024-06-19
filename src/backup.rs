use std::fs;
use std::path::Path;
use anyhow::anyhow;
use axum::body::Bytes;

pub fn generate_all_folders() ->Result<(),anyhow::Error>{
    let locations = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATION not set, please set in ENV or .env file");
    for loc in locations.split(":") {
        if !fs::metadata(loc).is_ok() {
            if std::env::var("GENERATE_DIRS").unwrap_or("false".to_string()).to_string() == "True".to_string(){
                fs::create_dir_all(loc)?
            }else{
                return Err(anyhow::anyhow!("Directory {} does not exist, Please set GENERATE_DIRS to True", loc));
            }
        }
    }
    Ok(())
}

pub fn sync_files() -> Result<(), anyhow::Error> {
    let binding = std::env::var("REPLICATION_LOCATIONS")?;
    let locations: Vec<&str> = binding.split(':').collect();

    for &loc2 in &locations {
        for &loc1 in &locations {
            if loc1 != loc2 {
                println!("Syncing {} to {}", loc1, loc2);
                sync_directory(loc1, loc2)?;
            }
        }
    }

    Ok(())
}

fn sync_directory(src: &str, dest: &str) -> anyhow::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = Path::new(dest).join(entry.file_name());

        if path.is_dir() {
            if !dest_path.exists() {
                fs::create_dir(&dest_path)?;
            }
            sync_directory(&path.to_string_lossy(), &dest_path.to_string_lossy())?;
        } else if path.is_file() {
            if !dest_path.exists() {
                fs::copy(&path, &dest_path)?;
            }
        }
    }
    Ok(())
}

pub async fn write_file(file_name: String, data: Bytes) -> Result<(), anyhow::Error> {
    let locations = std::env::var("REPLICATION_LOCATIONS").expect("REPLICATION_LOCATION not set, please set in ENV or .env file");
    for loc in locations.split(":") {
        fs::write(format!("{}/{}", loc, file_name), data.clone())?
    }
    Ok(())
}
