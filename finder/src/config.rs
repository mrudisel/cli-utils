use std::path::PathBuf;

use thread_pool::ThreadPool;

use super::Finder;
use super::pattern::{FileType, Pattern};


#[derive(Debug, Default)]
pub struct FinderConfig {
    pub root: Option<PathBuf>,
    pub patterns: Vec<Pattern>,
    pub file_types: Vec<FileType>,

    pool_size: Option<usize>,
}

impl FinderConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pool_size(&mut self, size: usize) -> Self {
        self.pool_size = Some(size);
        *self
    }

    pub fn with_pattern(&mut self, pattern: Pattern) -> Self {
        self.patterns.push(pattern);
        *self
    }

    pub fn with_patterns<V>(&mut self, patterns: V) -> Self
    where
        V: Into<Vec<Pattern>>
    {
        self.patterns.extend(patterns.into());
        *self
    }

    pub fn file_type(&mut self, file_type: FileType) -> Self {
        self.file_types.push(file_type);
        *self
    }

    pub fn file_types<V>(&mut self, file_types: V) -> Self
    where
        V: Into<Vec<FileType>>
    {
        self.file_types.extend(file_types.into());
        *self
    }

    pub fn clear_patterns(&mut self) -> Self {
        self.patterns.clear();
        *self
    }

    pub fn clear_file_types(&mut self) -> Self {
        self.file_types.clear();
        *self
    }

    pub fn build(self) -> Finder {
        self.into()
    }
}

impl<I: Into<PathBuf>> From<I> for FinderConfig {
    fn from(path: I) -> Self {
        Self {root: Some(path.into()), ..Default::default()}
    }
}

impl Into<Finder> for FinderConfig {
    fn into(self) -> Finder {
        Finder {
            root: self.root.unwrap_or_else(|| PathBuf::from("/")),
            config: self,
            pool: Some(ThreadPool::create_pool(self.pool_size)),
            ..Default::default()
        }
    }
}
