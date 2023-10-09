#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod db;
pub mod consts;
pub mod pixel;
pub mod comp;
pub mod qoi_errror;

use std::env;
use std::fs;
use tauri::State;
use std::path::Path;
use rusqlite::Error;
use std::ffi::OsStr;

use image::DynamicImage;

// own crate
use crate::comp:: Data;
use crate::db::Table;
use crate::db::DbFunctions;

const IMG_FOLDER_PATH: &str = r"C:\coding\RUST\image_compressor\src\app_imgs\";

fn create_img_folder() -> Result<(), std::io::Error>{
    fs::create_dir_all(IMG_FOLDER_PATH)?;
    Ok(())
}

fn file_name(file: &str) -> &OsStr {
    let file_name: &OsStr = match Path::new(file).file_name().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Invalid file path",)) {
        Ok(name) => name,
        Err(_) => panic!("Problem at reading file!")
    };
    file_name
}

#[tauri::command]
fn save_file_inside_db(app_db: State<'_, Table>, file: &str) -> Option<String> { 
    let _res: Result<(), String> = match app_db.save_file(file) { 
        Ok(()) => Ok(()),
        _ => panic!("Problem!")
    };

    // Checks path
    fs::create_dir_all(IMG_FOLDER_PATH).ok();

    // No type hinting, type is &OsStr
    let file_name = file_name(file);

    let combined_path = Path::new(IMG_FOLDER_PATH).join(file_name);
    
    // Copy the image to the assets folder.
    fs::copy(file, &combined_path).ok();


    // Send to frontend final path.
    let final_path: String = IMG_FOLDER_PATH.to_owned() + file_name.to_str().unwrap();
    Some(final_path)
}

#[tauri::command] 
fn compress(app_db: State<'_, Table>) {

    let file: Result<String, Error> = app_db.last_file();

    let img: DynamicImage = image::open(Path::new(file.as_ref().unwrap())).unwrap();
    let data: Data = Data { path: file.unwrap(), img: img };

    // create buffer for current data.
    let qoi_file: String = data.path.replace(&data.path[data.path.len() - 3..], "qoi");
    data.compress(qoi_file);

}

fn main() {

    env::set_var("RUST_BACKTRACE", "1");
    let app_db: Table;

    // Different scope, for 'temp_table_name' to be dropped
    {
        let temp_table_name: String = String::from("files");
        app_db = Table {
            table_name: temp_table_name.clone(), 
            create_query: format!("CREATE TABLE IF NOT EXISTS {} (
                file_path TEXT
            )", temp_table_name.clone()),
        }
    }

    // Creating DB for 'app' then handling errors.
    let _res_of_db: Result<(), Error> = match app_db.create_table() { 
        Ok(()) => Ok(()),
        _ => panic!("Problem as opening table!")
    }; 

    let _res_of_dir: Result<(), std::io::Error> = match create_img_folder() {
        Ok(()) => Ok(()),
        _ => panic!("Problem at opening img folder!")
    };


    tauri::Builder::default()
    .manage(app_db)
    .invoke_handler(tauri::generate_handler![save_file_inside_db, compress])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}