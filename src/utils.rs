use std::{fs, path::PathBuf};

use crate::error::KlsError;

#[allow(unused)]
pub trait StrUtil {
    fn substr(&self, pos: usize, len: usize) -> Self;
    fn substr_after(&self, post: usize) -> Self;
    fn repeat_to(&self, max_len: usize) -> Self;
    fn pad_start(&self, max_len: usize, c: char) -> Self;
    fn pad_end(&self, max_len: usize, c: char) -> Self;
    fn pad_start_str(&self, max_len: usize, s: &str) -> Self;
    fn pad_end_str(&self, max_len: usize, s: &str) -> Self;
}

#[allow(unused)]
pub trait PathUtil {
    fn kstr(&self) -> String;
    fn canonicalize_relative_to(&self, base: &Self) -> Result<Self, KlsError> where Self: Sized;
    fn kabsolute(&self) -> Result<Self, KlsError> where Self: Sized;
}

impl StrUtil for String {
    fn substr(&self, pos: usize, len: usize) -> Self {
        self.chars().skip(pos).take(len).collect()
    }

    fn substr_after(&self, pos: usize) -> Self {
        self.substr(pos, usize::MAX)
    }

    fn repeat_to(&self, max_len: usize) -> Self {
        if self.len() == 0 {
            self.clone()
        } else {
            self.repeat(max_len / self.len() + (max_len % self.len() != 0) as usize).substr(0, max_len)
        }
    }

    fn pad_start(&self, max_len: usize, c: char) -> Self {
        let mut result = String::from(self);
        if max_len > self.len() {
            result.insert_str(0, c.to_string().repeat(max_len - self.len()).as_str());
        }
        result
    }

    fn pad_end(&self, max_len: usize, c: char) -> Self {
        let mut result = String::from(self);
        if max_len > self.len() {
            result.push_str(c.to_string().repeat(max_len - self.len()).as_str());
        }
        result
    }

    fn pad_start_str(&self, max_len: usize, s: &str) -> Self {
        let mut result = String::from(self);
        if max_len > self.len() {
            result.insert_str(0, s.to_string().repeat_to(max_len - self.len()).as_str());
        }
        result
    }

    fn pad_end_str(&self, max_len: usize, s: &str) -> Self {
        let mut result = String::from(self);
        if max_len > self.len() {
            result.push_str(s.to_string().repeat_to(max_len - self.len()).as_str());
        }
        result
    }
}

impl PathUtil for PathBuf {
    fn kstr(&self) -> String {
        self.as_os_str().to_str().unwrap_or("invalid-path").to_string()
    }

    fn canonicalize_relative_to(&self, base: &PathBuf) -> Result<PathBuf, KlsError> {
        const ERR_CAN: &str = "Failed to canonicalize path";
        
        let path = if self.is_relative() {
            base.join(&self)
        } else {
            PathBuf::from(&self)
        };

        Ok(fs::canonicalize(path).map_err(|e| KlsError::E(ERR_CAN.to_string(), Box::new(e)))?)
    }

    fn kabsolute(&self) -> Result<Self, KlsError> {
        const ERR_PARENT: &str = "Failed to get symlink parent";
        const ERR_NAME: &str = "Failed to get symlink filename";
        const ERR_CAN: &str = "Failed to canonicalize path";
        //let base = base.canonicalize().map_err(|e| KlsError::E(ERR_CAN.to_string(), Box::new(e)))?;

        if ! self.is_symlink() {
            Ok(self.canonicalize().map_err(|e| KlsError::E(ERR_CAN.to_string(), Box::new(e)))?)
        } else {
            let sym_dir = self
                .parent()
                .ok_or_else(|| KlsError::S(ERR_PARENT.to_string()))?
                .canonicalize()
                .map_err(|e| KlsError::E(ERR_CAN.to_string(), Box::new(e)))?;

            let sym_path = sym_dir.join(self
                .file_name()
                .ok_or_else(|| KlsError::S(ERR_NAME.to_string()))?);

            Ok(sym_path)
        }
    }
}

