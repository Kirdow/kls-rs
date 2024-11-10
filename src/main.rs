use std::path::PathBuf;
use std::error::Error;
use std::env;
use colored::Colorize;

use files::FilesList;

mod files;

struct FormattedEntry<T> {
    pub mode: T,
    pub links: T,
    pub user: T,
    pub group: T,
    pub size: T,
    pub modified: T,
    pub name: T 
}

impl FormattedEntry<String> {
    pub fn new(entry: &files::FilesEntry, name: &str) -> Self {
        Self {
            mode: entry.get_mode_str(),
            links: format!("{}", entry.get_link_count()),
            user: match entry.get_user_str() {
                Err(_) => String::from("root"),
                Ok(p) => p
            },
            group: match entry.get_group_str() {
                Err(_) => String::from("root"),
                Ok(p) => p
            },
            size: format!("{}", entry.size),
            modified: entry.modified.clone(),
            name: name.to_string()
        }
    }

    fn cmp_set(c: &mut usize, n: usize) {
        if n > *c {
            *c = n;
        }
    }

    pub fn pad(list: Vec<FormattedEntry<String>>) -> Vec<FormattedEntry<String>> {
        let mut max_len = FormattedEntry::<usize> {
            mode: 0,
            links: 0,
            user: 0,
            group: 0,
            size: 0,
            modified: 0,
            name: 0
        };

        for entry in &list {
            Self::cmp_set(&mut max_len.mode, entry.mode.len());
            Self::cmp_set(&mut max_len.links, entry.links.len());
            Self::cmp_set(&mut max_len.user, entry.user.len());
            Self::cmp_set(&mut max_len.group, entry.group.len());
            Self::cmp_set(&mut max_len.size, entry.size.len());
            Self::cmp_set(&mut max_len.modified, entry.modified.len());
            Self::cmp_set(&mut max_len.name, entry.name.len());
        }

        list
            .into_iter()
            .map(|e| {
                let mut name = e.name;
                if e.mode.chars().nth(0).unwrap() == 'd' {
                    name = name.blue().bold().to_string();
                } else if e.mode.contains("x") {
                    name = name.green().bold().to_string();
                }
                Self {
                    mode: format!("{:>width$}", e.mode, width = max_len.mode),
                    links: format!("{:>width$}", e.links, width = max_len.links),
                    user: format!("{:>width$}", e.user, width = max_len.user),
                    group: format!("{:>width$}", e.group, width = max_len.group),
                    size: format!("{:>width$}", e.size, width = max_len.size),
                    modified: format!("{:>width$}", e.modified, width = max_len.modified),
                    name: name
                }
            })
            .collect()
    }
}

pub fn get_start_path() -> PathBuf {
    match env::args().nth(1) {
        None => PathBuf::from("./"),
        Some(s) => PathBuf::from(s)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = get_start_path();
    let files_list = FilesList::new(&path)?;

    let mut format_list: Vec<FormattedEntry<String>> = vec![];
    format_list.push(FormattedEntry::new(&files_list.dir, "."));
    if let Some(dir) = &files_list.up_dir {
        format_list.push(FormattedEntry::new(dir, ".."));
    }

    for entry in files_list.entries {
        if let Some(file_name) = entry.name() {
            format_list.push(FormattedEntry::new(&entry, file_name));
        }
    }

    let format_list = FormattedEntry::<String>::pad(format_list);

    println!("total {}", files_list.blocks);
    for entry in format_list {
        println!("{} {} {} {} {} {} {}", entry.mode, entry.links, entry.user, entry.group, entry.size, entry.modified, entry.name);
    }

    Ok(())
}

