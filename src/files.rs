use std::error::Error;
use std::path::PathBuf;
use std::{fmt, fs};
use chrono::{DateTime, Local, Duration, Datelike, Timelike, Utc};
use crate::error::KlsError;
use crate::params::Opts;
use crate::utils::{StrUtil, PathUtil};

pub enum FilesType {
    Dir(PathBuf),
    File(PathBuf),
    Sym(PathBuf, PathBuf)
}

impl FilesType {
    pub fn path(&self) -> &PathBuf {
        match self {
            FilesType::Dir(p) => &p,
            FilesType::File(p) => &p,
            FilesType::Sym(s, _) => &s
        }
    }
}

impl FilesType {
    pub fn canonicalize(&self) -> Result<FilesType, KlsError> {
        match self {
            FilesType::Dir(p) => Ok(FilesType::Dir(p.kabsolute()?)),
            FilesType::File(p) => Ok(FilesType::File(p.kabsolute()?)),
            FilesType::Sym(s, p) => Ok(FilesType::Sym(s.kabsolute()?, p.to_owned()))
        }
    }
}

impl fmt::Display for FilesType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilesType::Dir(p) => write!(f, "{}", p.kstr()),
            FilesType::File(p) => write!(f, "{}", p.kstr()),
            FilesType::Sym(s, p) => write!(f, "{} -> {}", s.kstr(), p.kstr())
        }
    }
}

pub struct FilesEntry {
    pub file_type: FilesType,
    perms: u16,
    pub size: u64,
    pub modified: String
}

impl fmt::Display for FilesEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (path_str, path_type) = match &self.file_type {
            FilesType::Dir(p) => (p.kstr(), "Dir"),
            FilesType::File(p) => (p.kstr(), "File"),
            FilesType::Sym(s, p) => (format!("{} -> {}", s.kstr(), p.kstr()), "Sym")
        };

        write!(f, "FilesEntry(\"{}\", {}, {:o}, {}, {})", path_str, path_type, self.perms, self.size, self.modified)
    }
}

impl FilesEntry {
    pub fn new_file(path: &PathBuf, meta: fs::Metadata) -> Self {
        Self::new(FilesType::File(path.to_owned()), meta)
    }

    pub fn new_dir(path: &PathBuf, meta: fs::Metadata) -> Self {
        Self::new(FilesType::Dir(path.to_owned()), meta)
    }

    pub fn new_sym(sym: &PathBuf, path: &PathBuf, meta: fs::Metadata) -> Self {
        Self::new(FilesType::Sym(sym.to_owned(), path.to_owned()), meta)
    }

    fn new(file_type: FilesType, meta: fs::Metadata) -> Self {
        let mode: u16;
        let size: u64;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            mode = meta.mode() as u16;
            size = meta.size();
        }
        #[cfg(not(unix))]
        {
            mode = 0o0777;
            size = 0;
        }

        let file_type = match file_type.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to canonicalize path \"{}\": {}", file_type, e);
                file_type
            }
        };

        Self {
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
                    let dt = DateTime::<Utc>::from(time);

                    let day = day.to_string().pad_start(2, ' ');
                    let month = match month {
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

                    let year = year.to_string();
                    let hour = hour.to_string().pad_start(2, '0');
                    let minute = minute.to_string().pad_start(2, '0');
                    let time = format!("{}:{}", hour, minute);

                    if dt <= six_months_ago {
                        format!("{} {} {}", month, day, year)
                    } else {
                        format!("{} {} {}", month, day, time)
                    }
                    
                }
            }
        }
    }

    pub fn path(&self) -> &PathBuf {
        self.file_type.path()
    }

    pub fn name(&self) -> Option<&str> {
        self.path().file_name().and_then(|os| os.to_str())
    }

    pub fn up_dir(&self) -> Option<PathBuf> {
        self.path().parent().map(|p| p.to_path_buf())
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

    fn split_mode(&self, section: u16) -> (bool, bool, bool) {
        let mode = self.perms >> (section * 3);

        (
            (mode & 0o4) != 0,
            (mode & 0o2) != 0,
            (mode & 0o1) != 0
        )
    }

    fn pick_mode(bits: &(bool, bool, bool), xc: &[u8; 2]) -> String {
        let rc= if bits.0 { 'r' } else { '-' };
        let wc= if bits.1 { 'w' } else { '-' };
        let xc = xc[bits.2 as usize] as char;

        format!("{}{}{}", rc, wc, xc)
    }

    pub fn get_mode_str(&self) -> String {
        let (uid_bit, gid_bit, sticky_bit) = self.split_mode(3);
        let owner = self.split_mode(2);
        let group = self.split_mode(1);
        let other = self.split_mode(0);

        format!("{}{}{}{}",
            match self.file_type {
                FilesType::File(_) => "-",
                FilesType::Dir(_) => "d",
                FilesType::Sym(_, _) => "l"
            },
            Self::pick_mode(&owner, if uid_bit { b"Ss" } else { b"-x" }),
            Self::pick_mode(&group, if gid_bit { b"Ss" } else { b"-x" }),
            Self::pick_mode(&other, if sticky_bit { b"Tt" } else { b"-x" })
        )
    }

    pub fn get_group_str(&self) -> Result<String, String> {
        let path = self.path();
        let meta = if path.is_symlink() {
            path.symlink_metadata()
        } else {
            path.metadata()
        };

        if let Ok(meta) = meta {
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
        let path = self.path();
        let meta = if path.is_symlink() {
            path.symlink_metadata()
        } else {
            path.metadata()
        };

        if let Ok(meta) = meta {
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
            match nix::sys::stat::lstat(self.path()) {
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
    pub fn new(path: &PathBuf, opts: &Opts) -> Result<Self, Box<dyn Error>> {
        let mut list: Vec<FilesEntry> = vec![];
        let self_entry = FilesEntry::new_dir(&path.to_path_buf(), path.metadata()?);

        let block_size = get_block_size();
        let mut blocks: i64 = 0;

        #[cfg(unix)]
        {
            if opts.all_files {
                blocks += match nix::sys::stat::lstat(path) {
                    Err(_) => 0,
                    Ok(p) => {
                        p.st_blocks
                    }
                };

                let parent_path = if let Some(parent) = self_entry.path().parent() {
                    parent
                } else {
                    self_entry.path().as_path()
                };

                blocks += match nix::sys::stat::lstat(parent_path) {
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
                match path.file_name().and_then(|p|p.to_str()) {
                    None => (),
                    Some(s) => {
                        if let Some(c) = s.chars().nth(0) {
                            if c == '.' && !opts.all_files {
                                continue;
                            }
                        }
                    }
                }


                let files_entry: FilesEntry = if path.is_symlink() {
                    FilesEntry::new_sym(&path.kabsolute()?, &std::fs::read_link(&path)?, meta)
                } else if path.is_dir() {
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
