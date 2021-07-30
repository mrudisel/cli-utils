use std::thread::{self, JoinHandle};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,

};

use super::task::{Task, Shape};
use super::cmd_pipe::CommandPipe;


#[derive(Clone)]
pub enum Command<D>
where
    D: Copy + Send + 'static
{
    New(Task<D>),
    Terminate,
}


pub struct Worker<D>
where
    D: Copy + Send + 'static
{
    // General id for the worker
    pub id: usize,
    busy_weak: Weak<AtomicBool>,

    handle: Option<JoinHandle<()>>,

    command_pipe: CommandPipe<D>,
    results_sender: Sender<D>,
}


impl<D> Worker<D>
where
    D: Copy + Send + 'static
{
    pub fn new(
        id: usize,
        command_pipe: CommandPipe<D>,
        results_sender: Sender<D>,
    ) -> Self {

        let cmd: CommandPipe<D> = command_pipe.clone();
        let results_sender_clone = results_sender.clone();

        let busy = Arc::new(AtomicBool::new(false));
        let busy_weak = Arc::downgrade(&busy);

        let handle = thread::spawn(move || {
            loop {
                let new_task: Task<D> = {
                    match cmd.receiver.recv() {
                        Ok(Command::New(job)) => job,
                        Ok(Command::Terminate) | _ => break
                    }
                };

                busy.store(true, Ordering::SeqCst);
                let task_result = new_task();
                busy.store(false, Ordering::SeqCst);

                if let Some(result) = task_result {
                    let (results_opt, new_jobs_opt) = result.get();

                    if let Some(new_jobs) = new_jobs_opt {
                        cmd.sender.send(new_jobs).expect("Command pipe disconnected");
                    }

                    if let Some(results) = results_opt {
                        match results {
                            Shape::Single(single) => {
                                results_sender_clone.send(single).expect("thread pool hung up")
                            },
                            Shape::Batch(batch) => {
                                for inner in batch {
                                    results_sender_clone.send(inner).expect("thread pool hung up");
                                }
                            },
                        }
                    }
                }
            }
        });

        Self {
            id,
            command_pipe,
            busy_weak,
            results_sender,
            handle: Some(handle),
        }
    }

    pub fn is_running(&self) -> bool {
        self.busy_weak.upgrade().is_some()
    }

    pub fn is_busy(&self) -> bool {
        match self.busy_weak.upgrade() {
            Some(busy) => busy.load(Ordering::SeqCst),
            None => false
        }
    }

    pub fn signal_termination(&self) {
        self.command_pipe.sender.send_termination().expect("");
    }

    pub fn join(&mut self) -> Option<()> {
        match self.handle.take() {
            Some(handle) => handle.join().ok(),
            None => None
        }
    }

    pub fn respawn_thread<F>(&mut self) -> Result<(), &'static str> {
        if self.handle.is_some() {
            Err("Thread is already running, terminate + .join before trying to respawn")
        }
        else {
            let cmd: CommandPipe<D> = self.command_pipe.clone();
            let results_sender_clone = self.results_sender.clone();

            let busy = Arc::new(AtomicBool::new(false));
            let busy_weak = Arc::downgrade(&busy);

            let handle = thread::spawn(move || {
                loop {
                    let new_job: Task<D> = {
                        match cmd.receiver.recv() {
                            Ok(Command::New(job)) => job,
                            Ok(Command::Terminate) | _ => break
                        }
                    };

                    busy.store(true, Ordering::SeqCst);
                    let job_result = new_job();
                    busy.store(false, Ordering::SeqCst);

                    if let Some(result) = job_result {
                        let (results_opt, new_jobs_opt) = result.get();

                        if let Some(new_jobs) = new_jobs_opt {
                            cmd.sender.send(new_jobs)
                                .expect("");
                        }

                        if let Some(results) = results_opt {
                            match results {
                                Shape::Single(single) => {
                                    results_sender_clone.send(single).expect("thread pool hung up")
                                },
                                Shape::Batch(batch) => {
                                    for inner in batch {
                                        results_sender_clone.send(inner).expect("");
                                    }
                                },
                            }
                        }
                    }
                }
            });

            self.handle = handle.into();
            self.busy_weak = busy_weak;

            Ok(())
        }
    }
}
