
use std::path::{Path, PathBuf};


use lazy_static::lazy_static;

use mime_guess;
use mime_guess::mime;

use regex::{Regex, Captures};

use sublime_fuzzy::FuzzySearch;

static DEFAULT_FUZZY_THRESHOLD: isize = 1;



#[derive(Debug, Clone)]
pub enum FileType {
    Any,
    Image,
    Video,
    Text,
    Code,
    Json,
}


impl FileType {
    pub fn to_pattern(&self) -> Pattern {
        self.clone().into()
    }

    pub fn matches_mime(&self, det_mime: &mime::Mime) -> bool {
        let name = det_mime.type_();

        match &self {
            FileType::Any => true,
            FileType::Image => name == "image",
            FileType::Video => name == "video",
            FileType::Text | FileType::Code | FileType::Json => name == "text",
            _ => false,
        }
    }

    pub fn matches_file(&self, path: &Path) -> bool {
        match mime_guess::from_path(path).first() {
            Some(detected_mime) => self.matches_mime(&detected_mime),
            _ => false,
        }
    }
}



#[derive(Debug, Clone)]
pub enum Pattern {
    Ext(String),
    Regex(Regex),
    Fuzzy(String, isize),
    FileType(FileType)
}

impl Pattern {
    pub fn ext<T>(ext: T) -> Self
    where
        T: Into<String>
    {
        Pattern::Ext(ext.into())
    }

    pub fn regex<T>(regex: T) -> Self
    where
        T: Into<Regex>
    {
        Pattern::Regex(regex.into())
    }

    pub fn fuzzy<T>(fuzzy: T, thresh: Option<isize>) -> Self
    where
        T: Into<String>
    {
        Pattern::Fuzzy(fuzzy.into(), thresh.unwrap_or(DEFAULT_FUZZY_THRESHOLD))
    }

    pub fn file_type(f_type: FileType) -> Self {
        Pattern::FileType(f_type)
    }


    pub fn matches(&self, path: &Path) -> bool {
        match &self {
            Pattern::Ext(ext) => path.ends_with(ext),
            Pattern::Regex(regex) => {
                path.to_str()
                    .map(|path_str| regex.is_match(path_str))
                    .unwrap_or(false)
            },
            Pattern::Fuzzy(fuzzy, thresh) => fuzzy_match_path(path, fuzzy.to_string()) > *thresh,
            Pattern::FileType(f_type) => f_type.matches_file(path),
        }
    }
}

impl From<FileType> for Pattern {
    fn from(file_type: FileType) -> Pattern {
        Pattern::FileType(file_type.clone())
    }
}
