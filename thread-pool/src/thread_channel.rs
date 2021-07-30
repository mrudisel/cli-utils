/*
use std::sync::{
    MutexGuard,
    mpsc::{channel, Sender, Receiver},
};

use aquire::aquire;

use super::job::{JobResult, Shape};
use super::job_channel::JobChannel;



#[derive(Clone)]
pub struct WorkerChannel<D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Send + 'static
{
    jobs: JobChannel<D, F>,

    result_sender: Sender<D>,
}


pub struct PoolChannel<D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Send + 'static
{
    jobs: JobChannel<D, F>,

    result_receiver: Receiver<D>,
}


impl<D, F> WorkerChannel<D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Send + 'static
{
    pub fn new(
        result_sender: Sender<D>,
        jobs: JobChannel<D, F>,
    ) -> Self {
        Self {result_sender, jobs}
    }

    pub fn send_result(&self, result: D) -> Result<(), D> {
        self.result_sender.send(result).map_err(|_| result)
    }

    pub fn send_job(&self, job: F) -> Result<(), F> {
        self.job_sender.send(job)
    }

    pub fn send_results(&self, results: Vec<D>) -> Result<(), Vec<D>> {
        let drain = results.drain(..);

        for result in drain {
            match self.result_sender.send(result) {
                Ok(_) => (),
                Err(_) => return Err(drain.collect()),
            }
        }

        Ok(())
    }

    pub fn send_jobs(&self, jobs: Vec<F>) -> Result<(), Vec<F>> {
        let drain = jobs.drain(..);

        let sender_lock = aquire!(self.job_sender);

        for job in drain {
            match sender_lock.send(job) {
                Ok(_) => (),
                Err(_) => return Err(drain.collect()),
            }
        }

        Ok(())
    }

    pub fn send(&self, job_result: JobResult<D, F>) -> Result<(), JobResult<D, F>> {
        let (results, jobs) = job_result.get();

        if let Some(results) = results {
            match send_shape(&self.result_sender, results) {
                Err(leftover) => return Err(JobResult::from(leftover.into(), jobs)),
                _ => (),
            }
        }

        if let Some(jobs) = jobs {
            let sender_lock = aquire!(self.job_sender);
            match send_shape(&sender_lock, jobs) {
                // If we make it here, there were either no results, or they all were
                // sent successfully, therefore we can assume None when returning
                // the unsent items
                Err(leftover) => return Err(JobResult::from(None, leftover.into())),
                _ => (),
            }
        }

        Ok(())
    }
}

impl<D, F> PoolChannel<D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Send + 'static
{
    pub fn new(
        result_receiver: Receiver<D>,
        jobs: JobChannel<D, F>,
    ) -> Self {
        Self {result_receiver, jobs}
    }

    pub fn send(&self, result: Job<D>) -> Result<(), mpsc::SendError<Job<D>>> {
        aquire!(self.job_sender).send(result)
    }

    pub fn recv(&self) -> Result<D, mpsc::RecvError> {
        self.result_receiver.recv()
    }

    pub fn try_recv(&self) -> Result<D, TryRecvError> {
        self.result_receiver.try_recv()
    }

    pub fn iter(&self) -> mpsc::Iter<'_, D> {
        self.result_receiver.iter()
    }

    pub fn try_iter(&self) -> mpsc::TryIter<'_, D> {
        self.result_receiver.try_iter()
    }

    pub fn recv_timeout(&self, timeout: Option<Duration>) -> Result<D, mpsc::RecvTimeoutError> {
        let timeout = timeout.unwrap_or(RECV_TIMEOUT);

        self.result_receiver.recv_timeout(timeout)
    }
}


fn send_shape<D>(sender: &Sender<D>, data: Shape<D>) -> Result<(), Shape<D>> {
    if let Shape::Single(single) = data {
        sender.send(single).map_err(|err| err.0.into())
    }
    else if let Shape::Batch(batch) = data {
        let drain = batch.drain(..);

        for item in drain {
            match sender.send(item) {
                Err(err) => return Err(Shape::Batch(drain.collect())),
                _ => ()
            }
        }

        Ok(())
    }
    else {
        panic!("Recieved an unknown shape");
    }
}


pub fn thread_channel<D>() -> (PoolEndpoint<D>, WorkerEndpoint<D>)
where
    D: Copy,
{
    let (job_send, job_recv) = queued_channel();
    let (result_send, result_recv) = mpsc::channel();

    let worker_end = WorkerEndpoint::new(work_send, work_recv.clone(), pool_send.clone());
    let pool_end = PoolEndpoint::new(pool_recv, work_recv, pool_send);

    (pool_end, worker_end)
}
*/
