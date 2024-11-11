use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use colored::{ColoredString, Colorize};

use crate::files::{self, FilesType};
use crate::params::Opts;
use crate::utils::{PathUtil, StrUtil};
use crate::colors::compute_color_for;

pub fn output(data: Vec<files::FilesList>, opts: &Opts) {
    let mut first = true;
    for list in &data {
        if !first {
            println!("");
        }

        if data.len() > 1 {
            println!("{}:", list.dir.name().unwrap());
        }

        if opts.long_format {
            output_one_list(list, opts);
        } else {
            output_short_format(list, opts);
        }

        first = false;
    }
}

fn get_formatted_list(list: &files::FilesList, opts: &Opts) -> Vec<FormattedEntry> {
    let mut format_list: Vec<FormattedEntry> = vec![];
    if opts.all_files {
        format_list.push(FormattedEntry::new(&list.dir, "."));
        if let Some(dir) = &list.up_dir {
            format_list.push(FormattedEntry::new(dir, ".."));
        } else {
            format_list.push(FormattedEntry::new(&list.dir, ".."));
        }
    }

    for entry in &list.entries {
        if let Some(file_name) = entry.name() {
            format_list.push(FormattedEntry::new(&entry, file_name));
        }
    }

    format_list
}

pub fn output_one_list(list: &files::FilesList, opts: &Opts) {
    let format_list = get_formatted_list(list, opts);
    let format_list = FormattedEntry::pad(format_list, opts);

    println!("total {}", list.blocks);
    for entry in format_list {
        println!("{} {} {} {} {} {} {}", entry.mode, entry.links, entry.user, entry.group, entry.size, entry.modified, entry.name);
    }
}

fn output_short_format(list: &files::FilesList, opts: &Opts) {
    let format_list = get_formatted_list(list, opts);

    let mut first = true;
    for entry in format_list {
        if !first {
            print!("  ");
        }

        print!("{}", entry.get_colored_name(opts));
        
        first = false;
    }
    println!();
}

enum FormattedFile {
    File,
    Dir,
    Sym
}

impl Clone for FormattedFile {
    fn clone(&self) -> Self {
        match self {
            FormattedFile::File => FormattedFile::File,
            FormattedFile::Dir => FormattedFile::Dir,
            FormattedFile::Sym => FormattedFile::Sym
        }
    }
}

struct FormattedEntry {
    pub mode: String,
    pub links: String,
    pub user: String,
    pub group: String,
    pub size: String,
    pub modified: String,
    pub name: String,
    pub sym: Option<((String, PathBuf), FormattedFile)>
}

struct CountedEntry {
    pub mode: usize,
    pub links: usize,
    pub user: usize,
    pub group: usize,
    pub size: usize,
    pub modified: usize
}

impl FormattedEntry {
    pub fn new(entry: &files::FilesEntry, name: &str) -> Self {
        Self {
            mode: entry.get_mode_str(),
            links: format!("{}", entry.get_link_count()),
user: match entry.get_user_str() {
                Err(_) => String::from("-"),
                Ok(p) => p
            },
            group: match entry.get_group_str() {
                Err(_) => String::from("-"),
                Ok(p) => p
            },
            size: format!("{}", entry.size),
            modified: entry.modified.clone(),
            name: name.to_string(),
            sym: match &entry.file_type {
                FilesType::Sym(s, p) => Self::get_relative_path(s, p),
                _ => None
            }
        }
    }

    fn get_deep_type(path: &PathBuf) -> FormattedFile {
        if path.is_symlink() {
            match std::fs::read_link(path) {
                Err(_) => {
                    eprintln!("Failed to read deep symlink: {}", path.kstr());
                    FormattedFile::Sym
                },
                Ok(path) => Self::get_deep_type(&path)
            }
        } else if path.is_file() {
            FormattedFile::File
        } else {
            FormattedFile::Dir
        }
    }

    fn get_relative_path(a: &PathBuf, b: &PathBuf) -> Option<((String, PathBuf), FormattedFile)> {
        let rel = b.canonicalize_relative_to(&a.parent()?.to_path_buf()).ok()?;
        return Some(((b.kstr(), rel.to_owned()), Self::get_deep_type(&rel)));
    }

    pub fn pad(list: Vec<FormattedEntry>, opts: &Opts) -> Vec<FormattedEntry> {
        let mut max_len = CountedEntry::new();
        for entry in &list {
            max_len.next(entry);
        }
        list
            .into_iter()
            .map(|e| max_len.apply(&e, opts))
            .collect()
    }

    pub fn get_colored_name(&self, opts: &Opts) -> String {
        let result = if let Some(((target, target_path), file_type)) = &self.sym {
            let name = self.name.bright_cyan().bold();
            if opts.long_format {
                let target = match file_type {
                    FormattedFile::File => {
                        let meta = if target_path.is_symlink() { target_path.symlink_metadata() } else { target_path.metadata() };
                        match meta {
                            Ok(meta) => {
                                if (meta.mode() & 0o111) != 0 {
                                    target.green().bold()
                                } else {
                                    //target.red().bold()
                                    ColoredString::from(target.to_string())
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to fetch target meta for: {}\n  Error: {}", target, e);
                                //target.magenta().bold()
                                target.green().bold()
                            }
                        }
                    },
                    FormattedFile::Dir => target.blue().bold(),
                    FormattedFile::Sym => target.bright_cyan().bold()
                };

                ColoredString::from(format!("{} -> {}",
                    name,
                    target))
            } else {
                name
            }
        } else if self.mode.chars().nth(0).unwrap() == 'd' {
            self.name.blue().bold()
        } else if self.mode.contains("x") {
            self.name.green().bold()
        } else {
            ColoredString::from(self.name.clone())
        };

        compute_color_for(result, &self.name.substr_after(self.name.rfind('.').map_or(0, |i|i+1))).to_string()
    }
}

impl CountedEntry {
    pub fn new() -> Self {
        Self {
            mode: 0,
            links: 0,
            user: 0,
            group: 0,
            size: 0,
            modified: 0
        }
    }

    fn cmp_set(c: &mut usize, n: usize) {
        if n > *c {
            *c = n;
        }
    }

    pub fn next(&mut self, entry: &FormattedEntry) {
        Self::cmp_set(&mut self.mode, entry.mode.len());
        Self::cmp_set(&mut self.links, entry.links.len());
        Self::cmp_set(&mut self.user, entry.user.len());
        Self::cmp_set(&mut self.group, entry.group.len());
        Self::cmp_set(&mut self.size, entry.size.len());
        Self::cmp_set(&mut self.modified, entry.modified.len());
    }

    pub fn apply(&self, e: &FormattedEntry, opts: &Opts) -> FormattedEntry {
        FormattedEntry {
            mode: format!("{:>width$}", e.mode, width = self.mode),
            links: format!("{:>width$}", e.links, width = self.links),
            user: format!("{:width$}", e.user, width = self.user),
            group: format!("{:width$}", e.group, width = self.group),
            size: format!("{:>width$}", e.size, width = self.size),
            modified: format!("{:>width$}", e.modified, width = self.modified),
            name: e.get_colored_name(opts),
            sym: e.sym.clone()
        }
    }
}
