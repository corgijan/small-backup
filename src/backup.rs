use std::fs;
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

pub fn sync_files() -> Result<(), anyhow::Error>{
    for loc2 in std::env::var("REPLICATION_LOCATIONS").unwrap().split(":") {
        for loc1 in std::env::var("REPLICATION_LOCATIONS").unwrap().split(":") {
            if loc1 != loc2 {
                println!("Syncing {} to {}", loc1, loc2);
                for file in fs::read_dir(loc1)? {
                    let file = file?;
                    // if file is not in loc2 write it to loc2
                    if !fs::metadata(format!("{}/{}", loc2, file.file_name().to_str().unwrap())).is_ok() {
                        let data = fs::read(format!("{}/{}", loc1, file.file_name().to_str().unwrap()))?;
                        fs::write(format!("{}/{}", loc2, file.file_name().to_str().unwrap()), data)?
                    }
                }
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
