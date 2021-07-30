use std::fs;
use std::fs::{DirEntry, Metadata, FileType};
use std::path::PathBuf;
use std::sync::Arc;

use thread_pool::{ThreadPool, Job, JobResult};

#[cfg(unix)]
use std::os::unix::fs::{DirEntryExt, FileTypeExt, FileExt, MetadataExt};


#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub meta: Metadata,
}

pub enum HandleFile {
    Dir(PathBuf),
    Match(FileType),
    Ignore,
}



pub fn get_dir_entries(path: PathBuf) -> Option<impl Iterator<Item = DirEntry>>
{
    match fs::read_dir(path) {
        Ok(contents) => Some(contents.filter_map(|entry| entry.ok())),
        _ => None
    }
}


pub fn recurse_find<F>(path: PathBuf, predicate: Arc<Box<F>>) -> impl FnOnce() -> Option<JobResult<Vec<FileInfo>>> + Sync + Send + 'static
where
    F: FnOnce(FileInfo) -> bool + Send + Sync + 'static
{
    let pred_clone = predicate.clone();

    return move || {
        let dir_entries = get_dir_entries(path)?;
        let mut found_files = vec![];

        for entry in dir_entries {
            let meta = match entry.metadata() {
                Ok(meta) => meta,
                _ => continue
            };

            let entry_path = entry.path();

            if meta.is_dir() {
                recurse_find(entry_path, pred_clone.clone());
                continue;
            }

            if meta.is_file() {
                let file_info = FileInfo {meta, path: entry_path};

                if pred_clone(file_info) {
                    found_files.push(file_info);
                }
            }
        }


        Some(JobResult::Result(found_files))
    };

}
