use crate::{
    pipe::{self, PoolPipe},
    worker::Worker,
    task::{Task, TaskHandler},
};

pub struct Pool<D> {
    pub n_threads: usize,

    workers: Vec<Worker>,
    pipe: PoolPipe<D>,

    results: Vec<D>,
}


impl<D> Pool<D>
where
    D: Clone + Send + 'static
{
    pub fn new() -> Self {
        Self::with_n_threads(num_cpus::get() - 1)
    }

    pub fn with_n_threads(n_threads: usize) -> Self {
        let (pool_pipe, worker_pipe) = pipe::pipe();

        let mut workers = Vec::with_capacity(n_threads);

        for id in 0..n_threads {
            workers.push(Worker::new(id, worker_pipe.clone()));
        }

        Self {workers, pipe: pool_pipe, n_threads, results: vec![]}
    }

    pub fn submit<F>(&self, closure: F) -> Result<(), Task<D>>
    where
        F: FnOnce(TaskHandler<D>) + Send + Sync + 'static
    {
        self.pipe.send_task(Box::new(closure))
    }


    pub fn compute<F>(&self, closure: F) -> Result<(), Task<D>>
    where
        F: FnOnce() -> D + Send + Sync + 'static
    {
        self.submit(move |handler| {
            handler.send_result(closure())
                .unwrap_or_else(|_| panic!("Failed to submit result"));
        })
    }

    pub fn join(mut self) -> Vec<D> {
        // Wait until the queue is empty
        self.pipe.wait();

        self.pipe.terminate(self.n_threads);

        for worker in self.workers.drain(..) {
            let worker_id = worker.id;
            if let Err(err) = worker.join() {
                println!("worker {} panicked: {:?}", worker_id, err);
            }
        }

        for res in self.pipe.iter_results() {
            self.results.push(res);
        }

        self.results
    }
}
