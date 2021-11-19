
use std::sync::mpsc::Sender;

use crate::pipe::WorkerPipe;



pub type Task<D> = Box<dyn FnOnce(TaskHandler<D>) + Send + Sync + 'static>;


pub enum Instruction<D> {
    NewTask(Task<D>),
    Terminate,
}

impl<D> Instruction<D> {
    /// Extracts the inner task, and panics if there isn't one.
    pub fn expect_task(self) -> Task<D> {
        match self {
            Instruction::NewTask(task) => task,
            _ => panic!("expected task, recieved terminate"),
        }
    }

    pub fn get_task(self) -> Option<Task<D>> {
        match self {
            Instruction::NewTask(task) => Some(task),
            _ => None
        }
    }
}


pub struct TaskHandler<'a, D> {
    worker_pipe: &'a WorkerPipe<D>,
    results_tx: &'a Sender<D>,
}

impl<'a, D> TaskHandler<'a, D> {
    pub fn new(worker_pipe: &'a WorkerPipe<D>, results_tx: &'a Sender<D>) -> Self {
        TaskHandler {worker_pipe, results_tx}
    }

    pub fn new_task<F>(&self, inner: F) -> Result<(), Task<D>>
    where
        F: FnOnce(TaskHandler<D>) + Send + Sync + 'static
    {
        self.worker_pipe.send_task(Box::new(inner))
    }

    pub fn send_result(&self, result: D) -> Result<(), D> {
        match self.results_tx.send(result) {
            Ok(_) => Ok(()),
            Err(send_err) => Err(send_err.0),
        }
    }

    pub fn send_results(&self, results: Vec<D>) -> Result<(), Vec<D>> {
        let remaining: Vec<D> = {
            results.into_iter().fold(vec![], |mut remaining, curr_result| {
                match self.results_tx.send(curr_result) {
                    Ok(_) => (),
                    Err(send_err) => remaining.push(send_err.0),
                }

                remaining
            })
        };

        if remaining.len() == 0 {
            Ok(())
        }
        else {
            Err(remaining)
        }
    }
}
