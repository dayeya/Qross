

use itertools::Itertools;
use rusqlite::{Connection, Result, Error};
use serde::{Deserialize, Serialize};

const DB_FILE_NAME: &str = "file_data.db";

#[derive(Deserialize, Serialize, Clone)]
pub struct Table {
    pub create_query: String,
    pub table_name: String,
}

pub trait DbFunctions {
    fn create_table(&self) -> Result<(), Error>;
    fn save_file(&self, file: &str) -> Result<(), Error>;
    fn fetch_all_files(&self) -> Result<Vec<String>, Error>;
}

// Impl block for all functions, REMEMBER when CON is out of scope the value is dropped (no need for .close)
impl DbFunctions for Table {

    // opens table in db file.
    fn create_table(&self) -> Result<(), Error> { 
        let con = Connection::open(DB_FILE_NAME)?;
        con.execute(&self.create_query, ())?;

        Ok(())
    }

    fn save_file(&self, file: &str) -> Result<(), Error> { 
        let con: Connection = Connection::open(DB_FILE_NAME)?;

        let query = format!("INSERT INTO {} (file_path) VALUES (?1)", &self.table_name);
        con.execute(&query, &[&file], )?;
        
        Ok(())
    }

    fn fetch_all_files(&self) -> Result<Vec<String>, Error> {
        let con = Connection::open(DB_FILE_NAME)?;

        let query = format!("SELECT file_path FROM {}", &self.table_name);
        
        let mut statement = con.prepare(&query, )?;
        let file_results: Vec<Result<String, Error>> = statement
        .query_map((), |r| {
            let get: Result<String, Error> = r.get(0);
            get
        })?.collect_vec();

        let files = file_results
        .iter()
        .map(|res| {
            match res { 
                Ok(s) => s.to_owned(),
                Err(e) => format!("ERROR {}", e),
            }
        })
        .collect_vec();
        
        Ok(files)
    }
}  