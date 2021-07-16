use std::{
    io,
    thread,
    time::Duration,
    path::PathBuf,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering}
};

static THREAD_COUNTER: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
static FILE_LIST: Arc<Vec<PathBuf>> = Arc::new(vec![]);




pub fn find_all(path: &PathBuf) -> io::Result<Vec<PathBuf>> {
    // If we got as file, return it as the only element in a vector
    if path.is_file() {
        FILE_LIST.fetch_add(1, Ordering::Relaxed);
        return Ok(vec![path.to_owned()]);
    }
    // Otherwise, if its not a directory, return an empty vector
    else if !path.is_dir() {
        return Ok(vec![]);
    }

    Ok(read_dir(path)?
        .filter_map(Result::ok) // Filter out any invalid DirEntries
        .map(|entry| recurse_path(&entry.path(), counter)) // Recursively check the inner paths
        .filter_map(Result::ok) // Filter out failed recursive calls
        .flat_map(|vec| vec) // then flatten to a 1d vector
        .collect()) // then collect
}

pub fn find_files() {

}
