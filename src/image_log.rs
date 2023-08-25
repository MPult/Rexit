//! This serves the purpose to keep track of images donwloaded to prevenet duplicates between runs

use cached::proc_macro::cached;
use std::hash;
use std::{path::PathBuf, fs::OpenOptions};
use std::io::Write;
use log::info;
use std::collections::HashSet;

/// Ensures that the log file exists and returns the initial reading.
pub fn init(out: PathBuf) -> std::string::String {
    info!("Initializing Image Log");

    if !out.join("imageLog.txt").exists() {
        std::fs::write(out.join("imageLog.txt"), "").expect("Error creating image log")
    }

    let image_log_contents = std::fs::read_to_string(out.join("imageLog.txt")).unwrap();

    return image_log_contents;
}

/// Gets the contents of the image log and returns it as a vector
#[cached]
pub fn read_image_log(out: PathBuf) -> Vec<String> {
    let binding = std::fs::read_to_string(out.join("imageLog.txt")).expect("Error reading Image Log");
    let image_log: Vec<String> = binding.split('\n').map(|line| line.to_string()).collect();

    return image_log
}

/// Writes to the image log
pub fn write_image_log(out: PathBuf, url: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(out.join("imageLog.txt"))
        .expect("Error writing to image log");

    writeln!(file, "{}", url).expect("Error writing to image log");
}

/// Checks if a entry exists in our log
pub fn check_image_log(out: PathBuf, url: String) -> bool{
    let binding = std::fs::read_to_string(out.join("imageLog.txt")).expect("Error reading Image Log");
    let image_log: Vec<String> = binding.split('\n').map(|line| line.to_string()).collect();
    
    let hash_set: HashSet<&String> = image_log.iter().collect();

    return hash_set.contains(&url);
}