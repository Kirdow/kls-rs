use std::path::PathBuf;
use std::env;

use files::FilesList;
use params::Params;
use error::KlsError;

mod files;
mod formatter;
mod params;
mod utils;
mod error;
mod colors;

pub fn get_start_path() -> PathBuf {
    match env::args().nth(1) {
        None => PathBuf::from("./"),
        Some(s) => PathBuf::from(s)
    }
}

fn main() -> Result<(), KlsError> {
    let params = Params::new()?;
    
    let mut files_lists: Vec<FilesList> = vec![];
    for path in &params.paths {
        let files_list = FilesList::new(path, &params.opts);

        if let Ok(files_list) = files_list {
            files_lists.push(files_list);
        } else {
            eprintln!("kls: cannot access '{}': No such file or directory.", path.to_str().unwrap_or("Unknown path"));
        }
    }

    formatter::output(files_lists, &params.opts);
    Ok(())
}

