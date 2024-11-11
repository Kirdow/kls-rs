use std::env;
use std::path::PathBuf;
use crate::utils::StrUtil;
use crate::error::KlsError;

pub struct Opts {
    pub long_format: bool,
    pub all_files: bool
}

pub struct Params {
    pub paths: Vec<PathBuf>,
    pub opts: Opts
}

impl Params {
    pub fn new() -> Result<Self, KlsError> {
        let mut params = Self {
            paths: vec![],
            opts: Opts {
                long_format: false,
                all_files: false
            }
        };

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            if arg.starts_with("--") {
                let arg = arg.substr_after(2);

                if arg == "long-format" {
                    params.opts.long_format = true;
                } else if arg == "-all" {
                    params.opts.all_files = true;
                } else {
                    return Err(KlsError::S(format!("Unknown argument: --{}", arg)));
                }
            } else if arg.starts_with("-") {
                let arg = arg.substr_after(1);

                if arg.contains('l') {
                    params.opts.long_format = true;
                }

                if arg.contains('a') {
                    params.opts.all_files = true;
                }
            } else {
                params.paths.push(PathBuf::from(arg));
            }
        }

        if params.paths.is_empty() {
            params.paths.push(PathBuf::from("./"));
        }

        Ok(params)
    }
}
