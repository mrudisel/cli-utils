use std::{
    io,
    fs,
    thread,
    thread::JoinHandle,
    path::PathBuf,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering}
};

#[allow(unused_imports)]
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

pub fn recurse_dir<U>(path: &PathBuf, thread_counter: &Arc<AtomicUsize>, update_fn: &U) -> io::Result<Vec<PathBuf>>
where U: Fn(usize) -> Option<usize> + Send + Copy + Sync + 'static
{
    let dir = fs::read_dir(&path)?;

    let mut thread_handles: Vec<JoinHandle<io::Result<Vec<PathBuf>>>> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for dir_entry_res in dir {
        if let Ok(dir_entry) = dir_entry_res {
            let meta = match dir_entry.metadata() {
                Ok(meta) => meta,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };

            let entry_path = dir_entry.path();

            if meta.is_dir() {
                match try_spawn_recurse_thread(&entry_path, thread_counter, update_fn) {
                    Some(handle) => thread_handles.push(handle),
                    None => files.append(&mut recurse_dir(&entry_path, thread_counter, update_fn)?),
                }
            }
            else if meta.is_file() {
                files.push(entry_path);
            }
        }
    }

    for handle in thread_handles {
        match handle.join() {
            Ok(inner_paths) => files.append(&mut inner_paths?),
            _ => continue,
        };
    }


    Ok(files)
}

fn try_spawn_recurse_thread<U>(path: &PathBuf, thread_counter: &Arc<AtomicUsize>, update_fn: &U) -> Option<JoinHandle<io::Result<Vec<PathBuf>>>>
where U: Fn(usize) -> Option<usize> + Send + Copy + Sync + 'static
{
    let can_spawn = thread_counter
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| update_fn(old))
        .is_ok();

    if can_spawn {
        let cloned_counter = thread_counter.clone();
        let cloned_update_fn = update_fn.clone();
        let cloned_path = path.clone();

        return Some(thread::spawn(move || {
            // println!("thread started looking at: {:?}", cloned_path);
            let results = recurse_dir(&cloned_path, &cloned_counter, &cloned_update_fn);
            cloned_counter.fetch_sub(1, Ordering::SeqCst);
            results
        }));
    }

    None
}
