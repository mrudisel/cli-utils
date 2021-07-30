use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::cmd_pipe::{CommandPipe, TrySendError};
use super::task::Task;
use super::worker::Worker;


pub struct ThreadPool<D>
where
    D: Copy + Send + 'static
{
    workers: Vec<Worker<D>>,

    job_channel: CommandPipe<D>,
    results_reciever: Receiver<D>,

    queue_size: Arc<AtomicUsize>,

    results: Option<Vec<D>>,
}


impl<D> ThreadPool<D>
where
    D: Copy + Send + 'static
{
    pub fn create_pool(max_workers_opt: Option<usize>) -> Self {
        let max_workers = max_workers_opt
            .map(|max_workers| max_workers.clamp(1, num_cpus::get()))
            .unwrap_or_else(num_cpus::get);

        let queue_size = Arc::new(AtomicUsize::new(0));

        let job_channel = CommandPipe::new(queue_size.clone());
        let (res_sender, res_receiver) = mpsc::channel();

        // Create and initialize the workers
        let mut workers: Vec<Worker<D>> = Vec::with_capacity(max_workers);
        for idx in 0..max_workers {
            workers.push(Worker::new(idx, job_channel.clone(), res_sender.clone()));
        }

        Self {
            workers,
            job_channel,
            queue_size,
            results_reciever: res_receiver,
            results: Some(vec![]),
        }

    }

    pub fn new() -> Self {
        Self::create_pool(None)
    }

    pub fn with_max_workers(max_workers: usize) -> Self {
        Self::create_pool(Some(max_workers))
    }

    pub fn worker_count(&self) -> usize {
        self.workers.iter()
            .filter(|worker| worker.is_running())
            .count()
    }

    pub fn num_busy_workers(&self) -> usize {
        self.workers.iter()
            .filter(|worker| worker.is_busy())
            .count()
    }

    pub fn spawn(&self, f: Task<D>) -> Result<usize, TrySendError> {
        self.job_channel.sender.send_single(f)?;

        Ok(self.job_channel.jobs_in_queue())
    }

    pub fn next_result(&mut self) -> Option<&'_ D> {
        let result = self.results_reciever.recv().ok()?;
        self.results.as_mut().expect("already consumed results").push(result);

        self.results.as_ref().map(|ref res| res.last()).unwrap_or(None)
    }

    pub fn n_results(&self) -> usize {
        self.results.as_ref().map(|ref res| res.len()).unwrap_or(0)
    }

    pub fn jobs_remaining(&self) -> usize {
        self.queue_size.load(Ordering::SeqCst)
    }

    pub fn join(&mut self) -> Vec<D> {
        for worker in &self.workers {
            worker.signal_termination();
        }

        for mut worker in self.workers.drain(..) {
            worker.join().expect("Error joining worker");
        }

        // Even though all the senders are dead, we still might have results in the buffer
        // so grab them if they exist.
        self.results.as_mut()
            .expect("results vec already consumed")
            .extend(self.results_reciever.iter());

        self.results.take().expect("already consumed results")
    }
}

impl<D> Default for ThreadPool<D>
where
    D: Copy + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/*
impl<'a, D, F> IntoIterator for &'a mut ThreadPool<D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<TaskResult<D, F>>,
    F: Clone + Send + 'static
{
    type Item = &'a D;
    type IntoIter = super::iter::ThreadPoolIter<'a, D, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}
*/

impl<D> Iterator for ThreadPool<D>
where
    D: Copy + Send + 'static
{
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        self.results_reciever.recv().ok()
    }
}

/*
impl <'a, T: 'a> IntoIterator for ThreadPool<'a, T>
where
    T: Copy + Clone + Send + Sync + 'static
{
    type Item = Rc<T>;
    type IntoIter = ThreadPoolIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ThreadPoolIter {pool: Rc::new(self), iter_idx: 0}
    }
}

impl <'a, T: 'a> IntoIterator for &ThreadPool<'a, T>
where
    T: Copy + Clone + Send + Sync + 'static
{
    type Item = Rc<T>;
    type IntoIter = ThreadPoolIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ThreadPoolIter {pool: Rc::new(*self), iter_idx: 0}
    }
}
*/
/*
impl<'a, T: 'a> Iterator for &'a ThreadPool<'a, T>
where
    T: Copy + Clone + Send + Sync + 'static
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let results = self.results.clone();

        if results.len() - self.iter_idx < 1 {
            self.await_next_result();
        }

        let next_val = results.get(self.iter_idx);

        if next_val.is_some() {
            self.iter_idx += 1;
        }

        next_val
        //None
    }
}
*/
/*
impl<T: Copy + Clone + Send + Sync + 'static> Iterator for ThreadPool<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.handle_result(self.reciever.recv().ok()?)
    }
}
*/
/*
impl<T: Copy + Clone + Send + Sync + 'static> Future for ThreadPool<T> {
    type Output = Vec<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.queue_size.load(Ordering::SeqCst) > 0 {
            let waker_clone = cx.waker().clone();
            self.waker = Some(Box::new(|| waker_clone.wake()));

            Poll::Pending
        }
        else {
            Poll::Ready(self.results)
        }
    }
}
*/
