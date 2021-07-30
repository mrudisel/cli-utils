mod iter;
mod task;
mod cmd_pipe;
mod pool;
mod worker;

pub use task::{Task, TaskResult};
pub use pool::ThreadPool;
pub use worker::Worker;


#[cfg(test)]
mod test {
    pub use super::*;

    use std::time::{Duration, Instant};




    #[test]
    fn basic_concurrency() {
        let mut thread_pool: ThreadPool<u64> = ThreadPool::create_pool(Some(10));
        let job_size: u64 = 10.min(thread_pool.worker_count() as u64);

        fn sleep_fac(dur: u64) -> &'static fn() -> Option<TaskResult<u64>> {
            &move || {
                std::thread::sleep(Duration::from_secs(dur));
                Some(TaskResult::from_result(dur))
            }
        }

        let start = Instant::now();
        for i in 0..job_size {
            thread_pool.spawn(move || {
                sleep(Duration::from_secs(i));

                Some(TaskResult::from_result(i))
            }).ok().expect("Could not submit job");
        }

        let results = thread_pool.join();
        let dur = Instant::now().duration_since(start);

        // Give a little bit of a buffer (10%) to account for threading overhead
        let max_dur = ((job_size as f32) * 1.1) as u64;

        assert!(dur.as_secs() < max_dur, "Tests took too long, concurrency may be broken");

        assert_eq!(0, thread_pool.worker_count(), "Threads not terminated correctly");

        assert_eq!(results, (0..job_size).collect::<Vec<u64>>());
    }

    /*
    #[test]
    fn repeat_jobs() {
        let mut thread_pool: ThreadPool<u64> = ThreadPool::create_pool(Some(10));
        let job_size: u64 = 10.min(thread_pool.worker_count() as u64);

        let start = Instant::now();

        for i in 0..job_size {
            thread_pool.spawn(move || {
                sleep(Duration::from_secs(i));

                if i % 2 == 0 {
                    let new_job = move || {
                        sleep(Duration::from_secs(i));
                        Some(JobResult::from_result(i))
                    };

                    Some(JobResult::from_job(Box::new(new_job)))
                }
                else {
                    Some(JobResult::from_result(i))
                }
            }).ok().expect("Could not submit job");
        }

        let results = thread_pool.join();

        let dur = Instant::now().duration_since(start);

        assert!(dur.as_secs() > job_size, "Tests finished too quickly, repeat jobs failed.");

        assert_eq!(0, thread_pool.worker_count(), "Threads not terminated correctly");

        assert_eq!(results.len(), job_size as usize, "Job count did not match results count");
    }
    */
    /*
    #[test]
    fn iter() {
        let mut thread_pool: ThreadPool<u64, _> = ThreadPool::create_pool(Some(10));
        let job_size: u64 = 10.min(thread_pool.worker_count() as u64);

        println!("Pool created with {} workers", job_size);

        for i in 0..job_size {
            thread_pool.spawn(move || {
                println!("sleeping for {} seconds", i);
                sleep(Duration::from_secs(i));

                if i % 2 == 0 {
                    let new_job = move || {
                        sleep(Duration::from_secs(i));
                        Some(JobResult::from_result(i))
                    };

                    Some(JobResult::from_job(new_job))
                }
                else {
                    Some(JobResult::from_result(i))
                }
            }).ok().expect("Could not submit job");

            println!("iter: {}, jobs_remaining {}", i, thread_pool.jobs_remaining());
        }

        println!("Jobs submitted, iterating over results");

        let mut results = Vec::new();

        let iter_start = Instant::now();
        // A random, big number that we can .min from.
        let mut min_dur = Duration::from_secs(10000);
        for result in thread_pool {
            min_dur = min_dur.min(Instant::now().duration_since(iter_start));
            println!("result recieved {}", result);
            results.push(result);
        }

        //let final_results = thread_pool.join();

        assert_eq!(results.len(), job_size as usize, "Iterating did not yield the same number of results");
    }
    */
}
