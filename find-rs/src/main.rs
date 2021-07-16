use std::{
    io::{Write, stdout},
    thread,
    time::{Duration, Instant},
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering}
};

mod cli;



fn main() {
    let args = match cli::parse_cli() {
        Ok(args) => args,
        Err(err) => panic!("{}", err),
    };

    if args.verbose {
        println!("{}", args);
    }

    let thread_counter = Arc::new(AtomicUsize::new(0));
    let file_counter = Arc::new(AtomicUsize::new(0));



}
