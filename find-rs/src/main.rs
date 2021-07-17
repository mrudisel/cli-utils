use std::{
    time::Instant,
    sync::Arc,
    sync::atomic::AtomicUsize,
};

mod cli;
mod lib;


fn main() {
    let args = match cli::parse_cli() {
        Ok(args) => args,
        Err(err) => panic!("{}", err),
    };

    if args.verbose {
        println!("{}", args);
    }

    let thread_counter = Arc::new(AtomicUsize::new(0));

    let update_fn = {
        let max_threads = args.worker_threads.clone();
        move |old_val: usize| {
            let updated_val = (old_val + 1).min(max_threads);

            if updated_val == max_threads {None}
            else {Some(updated_val)}
        }
    };

    let start = Instant::now();
    let results = match lib::recurse_dir(&args.root, &thread_counter, &update_fn) {
        Ok(res) => res,
        Err(err) => panic!("{}", err),
    };

    let elapsed = Instant::elapsed(&start);
    let capped_range = results.len().min(100);

    for i in 1..capped_range {
        println!("{:?}", results[i]);
    }

    println!("Found {} files in {:?}", results.len(), elapsed);

    for _ in 1..20 {
        println!("");
    }

    /*
    for file in &results {
        println!("{:?}", file);
    }
    */
}
