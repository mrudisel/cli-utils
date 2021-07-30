use std::sync::{
    Arc, Mutex, MutexGuard, TryLockError,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender},
};

use super::task::{Task, Shape};
use super::worker::Command;

use aquire::{try_aquire, aquire};

#[derive(Debug, Clone)]
pub enum TrySendError {
    WouldBlock,
    SendError,
}


#[derive(Clone)]
pub struct CommandSender<D>
where
    D: Copy + Send + 'static
{
    sender: Arc<Mutex<Sender<Command<D>>>>,
    queue_size: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct CommandReceiver<D>
where
    D: Copy + Send + 'static
{
    receiver: Arc<Mutex<Receiver<Command<D>>>>,
    queue_size: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct CommandPipe<D>
where
    D: Copy + Send + 'static
{
    pub sender: CommandSender<D>,
    pub receiver: CommandReceiver<D>,

    queue_size: Arc<AtomicUsize>,
}


impl<D> CommandSender<D>
where
    D: Copy + Send + 'static{
    fn new(sender: Arc<Mutex<Sender<Command<D>>>>, queue_size: Arc<AtomicUsize>) -> Self {
        Self {queue_size, sender}
    }

    fn send_many_inner(
        &self,
        locked_sender: MutexGuard<Sender<Command<D>>>,
        jobs: Vec<Task<D>>
    ) -> Result<(), TrySendError> {
        let n_jobs = jobs.len();

        let mut jobs_sent = 0;
        for job in jobs {
            match locked_sender.send(Command::New(job)) {
                Ok(()) => jobs_sent += 1,
                Err(_) => break,
            }
        }

        self.queue_size.fetch_add(jobs_sent, Ordering::SeqCst);

        if n_jobs == jobs_sent {
            Ok(())
        }
        else {
            Err(TrySendError::SendError)
        }
    }

    pub fn send_termination(&self) -> Result<(), TrySendError> {
        aquire!(self.sender)
            .send(Command::Terminate)
            .map_err(|_| TrySendError::SendError)
    }

    pub fn send_single(&self, job: Task<D>) -> Result<(), TrySendError> {
        aquire!(self.sender)
            .send(Command::New(job))
            .map_err(|_| TrySendError::SendError)?;

        self.queue_size.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    pub fn send_many(&self, jobs: Vec<Task<D>>) -> Result<(), TrySendError> {
        self.send_many_inner(aquire!(self.sender), jobs)
    }

    pub fn send(&self, jobs: Shape<Task<D>>) -> Result<(), TrySendError> {
        match jobs {
            Shape::Single(job) => self.send_single(job),
            Shape::Batch(jobs) => self.send_many(jobs),
        }
    }

    pub fn try_send_single(&self, job: Task<D>) -> Result<(), TrySendError> {
        try_aquire!(&self.sender)
            .map_err(|_| TrySendError::WouldBlock)?
            .send(Command::New(job))
            .map_err(|_| TrySendError::SendError)?;

        self.queue_size.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    pub fn try_send_many(&self, jobs: Vec<Task<D>>) -> Result<(), TrySendError> {
        let locked = try_aquire!(&self.sender).map_err(|_| TrySendError::WouldBlock)?;

        self.send_many_inner(locked, jobs)
    }

    pub fn try_send(&self, jobs: Shape<Task<D>>) -> Result<(), TrySendError> {
        match jobs {
            Shape::Single(job) => self.try_send_single(job).map_err(|err| err.into()),
            Shape::Batch(jobs) => self.try_send_many(jobs).map_err(|err| err.into()),
        }
    }
}


impl<D> CommandReceiver<D>
where
    D: Copy + Send + 'static
{
    fn new(receiver: Arc<Mutex<Receiver<Command<D>>>>, queue_size: Arc<AtomicUsize>) -> Self {
        Self {receiver, queue_size}
    }

    pub fn recv(&self) -> Result<Command<D>, ()> {
        let result = aquire!(self.receiver).recv().map_err(|_| ())?;

        self.queue_size.fetch_sub(1, Ordering::SeqCst);

        Ok(result)
    }

    pub fn try_recv(&self) -> Result<Command<D>, ()> {
        let result = try_aquire!(&self.receiver)
            .map_err(|_| ())?
            .try_recv()
            .map_err(|_| ())?;

        self.queue_size.fetch_sub(1, Ordering::SeqCst);

        Ok(result)
    }
}


impl<D> CommandPipe<D>
where
    D: Copy + Send + 'static
{
    pub fn new(queue_size: Arc<AtomicUsize>) -> Self {
        let (sender, receiver) = mpsc::channel();

        let sender = Arc::new(Mutex::new(sender));
        let receiver = Arc::new(Mutex::new(receiver));

        Self {
            sender: CommandSender::new(sender, queue_size.clone()),
            receiver: CommandReceiver::new(receiver, queue_size.clone()),
            queue_size,
        }
    }

    pub fn jobs_in_queue(&self) -> usize {
        self.queue_size.load(Ordering::SeqCst)
    }

    pub fn jobs_in_queue_relaxed(&self) -> usize {
        self.queue_size.load(Ordering::Relaxed)
    }
}
