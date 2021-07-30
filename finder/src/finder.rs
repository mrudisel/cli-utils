use std::path::PathBuf;
use std::sync::Arc;

use thread_pool::ThreadPool;

pub mod config;
pub mod pattern;
mod walk;

use config::FinderConfig;
use pattern::{FileType, Pattern};


#[derive(Debug, Default)]
pub struct Finder {
    root: PathBuf,
    config: FinderConfig,
    pool: Option<ThreadPool<PathBuf>>,
}


impl Finder {
    pub fn from_new_root(&mut self, root: PathBuf) -> Self {
        self.root = root;
        *self
    }

    pub fn add_pattern(&mut self, pattern: Pattern) -> Self {
        self.config.with_pattern(pattern);
        *self
    }

    pub fn add_patterns<L>(&mut self, patterns: L) -> Self
    where
        L: Into<Vec<Pattern>>
    {
        self.config.with_patterns(patterns);
        *self
    }

    pub fn clear_patterns(&mut self) -> Self {
        self.config.clear_patterns();
        *self
    }

    pub fn file_type(&mut self, file_type: FileType) -> Self {
        self.config.file_type(file_type);
        *self
    }

    pub fn file_types<L>(&mut self, file_types: L) -> Self
    where
        L: Into<Vec<FileType>>
    {
        self.config.file_types(file_types);
        *self
    }

    pub fn clear_file_types(&mut self) -> Self {
        self.config.clear_file_types();
        *self
    }

    pub fn start(&mut self) {
        let pool = Arc::new(self.pool.expect("Thread pool not initialized"));

    }

    pub fn collect(&mut self) {

    }




}
