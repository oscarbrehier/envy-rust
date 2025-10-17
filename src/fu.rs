// file utils

use std::fs;

pub fn backup_file(path: &str) -> Result<(), std::io::Error> {

    let content = fs::read_to_string(path)?;
    let backup_path = format!("{}.bak", path);

    fs::write(&backup_path, content)?;

    println!("Backup of {} located at {}", path, &backup_path);

    Ok(())

}

pub fn write_to_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    
    let temp_path = format!("{}.tmp", path);

    backup_file(path).expect("env backup failed");
    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, path)?;

    Ok(())

}