use std::error::Error;
use std::path::PathBuf;
use std::{fmt, fs};
use chrono::{DateTime, Local, Duration, Datelike, Timelike, Utc};

pub enum FilesType {
    Dir,
    File
}

impl FilesType {
    pub fn conditional_text<'a>(&self, dir: &'a str, file: &'a str) -> &'a str {
        match self {
            FilesType::Dir => dir,
            FilesType::File => file
        }
    }
}

impl fmt::Display for FilesType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilesType::Dir => write!(f, "{}", "Dir"),
            FilesType::File => write!(f, "{}", "File")
        }
    }
}

pub struct FilesEntry {
    pub path: PathBuf,
    pub file_type: FilesType,
    perms: u16,
    pub size: u64,
    pub modified: String
}

impl fmt::Display for FilesEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FilesEntry(\"{}\", {})", self.path.as_os_str().to_str().unwrap_or("Unknown Path"), self.file_type)
    }
}

impl FilesEntry {
    pub fn new_file(path: &PathBuf, meta: fs::Metadata) -> Self {
        Self::new(path.to_owned(), FilesType::File, meta)
    }

    pub fn new_dir(path: &PathBuf, meta: fs::Metadata) -> Self {
        Self::new(path.to_owned(), FilesType::Dir, meta)
    }

    fn new(path: PathBuf, file_type: FilesType, meta: fs::Metadata) -> Self {
        let perms = meta.permissions();
        let mut mode: u16 = 0o777;
        let mut size: u64 = 0;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            use std::os::unix::fs::MetadataExt;
            mode = perms.mode() as u16;
            size = meta.size();
        }

        let path = match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to canonicalize path \"{}\": {}", path.as_path().to_str().unwrap_or("Unknown Path"), e);
                path
            }
        };

        Self {
            path,
            file_type,
            perms: mode,
            size,
            modified: match meta.modified() {
                Err(_) => String::from("Jan 01 1970"),
                Ok(time) => {
                    let dt: DateTime<Local> = time.into();

                    let year = dt.year();
                    let month = dt.month();
                    let day = dt.day();
                    let hour = dt.hour();
                    let minute = dt.minute();

                    let current_time = Utc::now();
                    let six_months_ago = current_time - Duration::days(180);
                    let time = DateTime::<Utc>::from(time);

                    let day_str = if day < 10 { format!(" {}", day) } else { format!("{}", day) };
                    let month_str = match month {
                        1 => "Jan",
                        2 => "Feb",
                        3 => "Mar",
                        4 => "Apr",
                        5 => "May",
                        6 => "Jun",
                        7 => "Jul",
                        8 => "Aug",
                        9 => "Sep",
                        10 => "Oct",
                        11 => "Nov",
                        12 => "Dec",
                        _ => "Non"
                    };

                    let year_str = format!("{} ", year);
                    let hour = if hour < 10 { format!("0{}", hour) } else { format!("{}", hour) };
                    let minute = if minute < 10 { format!("0{}", minute) } else { format!("{}", minute) };
                    let time_str = format!("{}:{}", hour, minute);

                    if time <= six_months_ago {
                        format!("{} {} {}", month_str, day_str, year_str)
                    } else {
                        format!("{} {} {}", month_str, day_str, time_str)
                    }
                    
                }
            }
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.path.file_name().and_then(|os| os.to_str())
    }

    pub fn up_dir(&self) -> Option<PathBuf> {
        self.path.parent().map(|p| p.to_path_buf())
    }

    pub fn up_entry(&self) -> Option<FilesEntry> {
        self.up_dir().and_then(|p| {
            let meta = match p.metadata() {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("Failed to fetch parent metadata for \"{}\": {}", p.as_os_str().to_str().unwrap_or("Unknown Path"), e);
                    None
                }
            };

            if let Some(meta) = meta {
                Some(FilesEntry::new_dir(&p, meta))
            } else {
                None
            }
        })
    }

    fn mode_char(&self, i: u16, active_str: char) -> char {
        if self.perms & (1 << i) != 0 {
            active_str
        } else {
            '-'
        }
    }

    pub fn get_mode_str(&self) -> String {
        let mut s = String::new();
        s.push_str(self.file_type.conditional_text("d", "-"));
        let mut i = 0;
        let trio = "rwx";
        while i < 9 {
            s.push(self.mode_char(8 - i, trio.chars().nth((i as usize) % trio.len()).unwrap_or('-')));
            i += 1;
        }
        s
    }

    pub fn get_group_str(&self) -> Result<String, String> {
        if let Ok(meta) = self.path.metadata() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                use nix::unistd::{Gid, Group};

                let gid = Gid::from_raw(meta.gid());
                let group = Group::from_gid(gid).expect("Group fetch failed").expect("Group not found");

                Ok(group.name)
            }
            #[cfg(not(unix))]
            {
                Ok(String::new())
            }
        } else {
            Ok(String::new())
        }
    }

    pub fn get_user_str(&self) -> Result<String, String> {
        if let Ok(meta) = self.path.metadata() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                use nix::unistd::{Uid, User};

                let uid = Uid::from_raw(meta.uid());
                let user = User::from_uid(uid).expect("User fetch failed").expect("User not found");

                Ok(user.name)
            }
            #[cfg(not(unix))]
            {
                Ok(String::new())
            }
        } else {
            Ok(String::new())
        }
    }

    pub fn get_link_count(&self) -> u64 {
        #[cfg(unix)]
        {
            match nix::sys::stat::stat(&self.path) {
                Err(_) => 0,
                Ok(p) => p.st_nlink
            }
        }
        #[cfg(not(unix))]
        {
            0
        }
    }

}

fn get_block_size() -> i64 {
    fn get_env_size(s: &str) -> Option<i64> {
        match std::env::var(s) {
            Err(_) => None,
            Ok(s) => {
                match s.parse::<i64>() {
                    Err(_) => None,
                    Ok(p) => Some(p)
                }
            }
        }
    }

    get_env_size("LS_BLOCK_SIZE")
        .or_else(|| get_env_size("BLOCK_SIZE"))
        .or_else(|| get_env_size("BLOCKSIZE"))
        .or_else(|| get_env_size("POSIXLY_CORRECT"))
        .unwrap_or(1024)
}

pub struct FilesList {
    pub entries: Vec<FilesEntry>,
    pub dir: FilesEntry,
    pub up_dir: Option<FilesEntry>,
    pub blocks: i64 
}

impl FilesList {
    pub fn new(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut list: Vec<FilesEntry> = vec![];
        let self_entry = FilesEntry::new_dir(&path.to_path_buf(), path.metadata()?);

        let block_size = get_block_size();
        let mut blocks: i64 = 0;

        #[cfg(unix)]
        {
            blocks += match nix::sys::stat::lstat(path) {
                Err(_) => 0,
                Ok(p) => {
                    p.st_blocks
                }
            };
            if let Some(parent) = self_entry.path.parent() {
                blocks += match nix::sys::stat::lstat(parent) {
                    Err(_) => 0,
                    Ok(p) => {
                        p.st_blocks
                    }
                };
            }
        }

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                let path = entry.path();
                let files_entry: FilesEntry = if path.is_dir() {
                    FilesEntry::new_dir(&path, meta)
                } else {
                    FilesEntry::new_file(&path, meta)
                };

                #[cfg(unix)]
                {
                    blocks += match nix::sys::stat::lstat(&path) {
                        Err(_) => 0,
                        Ok(p) => {
                            p.st_blocks
                        }
                    }; 
                }

                list.push(files_entry);
            }
        }

        let up_entry = self_entry.up_entry();

        let mut result = Self {
            entries: list,
            dir: self_entry,
            up_dir: up_entry,
            blocks: (blocks * 512) / block_size
        };

        result.sort();

        Ok(result)
    }

    fn sort(&mut self) {
        self.entries.sort_by_key(|key| {
            let name = key.name().unwrap_or(".").to_lowercase();
            name.replace(".", "")
            //format!("{}{}", key.file_type.conditional_text(" ", ""), name)
        });
    }
}
