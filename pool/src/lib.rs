mod pipe;
pub mod pool;
pub mod task;
pub mod worker;

pub use crate::pool::Pool;
pub use task::TaskHandler;
pub use worker::Worker;

#[macro_export]
macro_rules! aquire {
    ($lock:expr) => {
        match $lock.lock() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        }
    };
}

#[macro_export]
macro_rules! aquire_write {
    ($lock:expr) => {
        match $lock.write() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        }
    };
}

#[macro_export]
macro_rules! aquire_read {
    ($lock:expr) => {
        match $lock.read() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        }
    };
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};
    use std::sync::Mutex;

    use lazy_static::lazy_static;

    use crate::*;

    lazy_static! {
        static ref SEQ_TEST: Mutex<()> = Mutex::new(());
    }

    /*
    #[test]
    fn basic_concurrency() {
        let _lock = SEQ_TEST.lock();

        let thread_pool: Pool<u64> = Pool::with_n_threads(10);

        let job_size: u64 = 10.min(thread_pool.n_threads as u64);

        let start = Instant::now();
        for i in 0..job_size {
            thread_pool.submit(move |handler| {
                thread::sleep(Duration::from_secs(i));
                handler.send_result(i).expect("could not send result");
            }).ok().expect("Could not submit job");
        }

        let results = thread_pool.join();
        let dur = Instant::now().duration_since(start);

        // Give a little bit of a buffer (10%) to account for threading overhead
        let max_dur = ((job_size as f32) * 1.1) as u64;

        assert!(dur.as_secs() < max_dur, "Tests took too long, concurrency may be broken");

        assert_eq!(results, (0..job_size).collect::<Vec<u64>>());
    }
    */
    
    #[test]
    fn resubmit_tasks() {
        let _lock = SEQ_TEST.lock();

        let thread_pool: Pool<u64> = Pool::with_n_threads(3);

        let job_size: u64 = 3.min(thread_pool.n_threads as u64);

        fn sleeper(handler: TaskHandler<u64>, i: u64) {
            if i == 0 {
                println!("Finished sleeping, bailing");
                handler.send_result(i).ok().expect("could not submit results");
                return;
            }

            println!("Sleeping for {} seconds", i);
            thread::sleep(Duration::from_secs(i));

            handler.send_result(i)
                .ok().expect("could not submit results");

            handler.new_task(move |h| {
                println!("Starting inner job with {} secs", i - 1);
                sleeper(h, i - 1)
            }).ok().expect("Could not submit inner task");
        }

        let start = Instant::now();
        for i in 0..job_size {
            thread_pool.submit(move |handler| sleeper(handler, i))
                .ok().expect("Could not submit job");
        }

        let results = thread_pool.join();
        let dur = Instant::now().duration_since(start);

        // Give a little bit of a buffer (10%) to account for threading overhead
        let max_dur = ((job_size as f32) * 1.1) as u64;

        assert!(dur.as_secs() < max_dur, "Tests took too long, concurrency may be broken");

        assert_eq!(results, (1..job_size).collect::<Vec<u64>>());
    }

}
