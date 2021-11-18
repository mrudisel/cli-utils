
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvError, TryRecvError, Sender};

use crate::aquire;
use crate::task::{Instruction, Task};


#[derive(Debug)]
pub struct PoolPipe<D> {
    inst_tx: Sender<Instruction<D>>,

    queue_size: Arc<AtomicUsize>,
    on_compl: Arc<(Mutex<()>, Condvar)>,
}


impl<D> PoolPipe<D> {
    pub fn send_task(&self, task: Task<D>) -> Result<(), Task<D>> {
        match self.inst_tx.send(Instruction::NewTask(task)) {
            Ok(_) => {
                let new_count = self.queue_size.fetch_add(1, Ordering::SeqCst);
                println!("sent (pool), {} -> {}", new_count - 1, new_count);
                Ok(())
            },
            Err(send_err) => {
                match send_err.0 {
                    Instruction::NewTask(inner) => Err(inner),
                    _ => panic!("terminate recieved in send_task SendError"),
                }
            }
        }
    }

    pub fn wait(&self) {
        let mut locked = aquire!(self.on_compl.0);

        while self.queue_size.load(Ordering::SeqCst) != 0 {
            locked = match self.on_compl.1.wait(locked) {
                Ok(l) => l,
                Err(poisoned) => poisoned.into_inner(),
            };

            println!("{} jobs remaining", self.queue_size.load(Ordering::SeqCst));
        }
    }

    pub fn terminate(&self, n_threads: usize) {
        for _ in 0..n_threads {
            match self.inst_tx.send(Instruction::Terminate) {
                Ok(_) => (),
                Err(_) => return
            }
        }
    }
}


#[derive(Debug)]
pub struct WorkerPipe<D> {
    inst_tx: Sender<Instruction<D>>,
    inst_rx: Arc<Mutex<Receiver<Instruction<D>>>>,

    queue_size: Arc<AtomicUsize>,
    on_compl: Arc<(Mutex<()>, Condvar)>,
}


impl<D> WorkerPipe<D> {
    pub fn recv_inst(&self) -> Result<Instruction<D>, RecvError> {
        let inst = aquire!(self.inst_rx).recv()?;

        let new = self.queue_size.fetch_sub(1, Ordering::SeqCst);

        println!("recv, {} -> {}", new + 1, new);

        Ok(inst)
    }

    pub fn send_task(&self, task: Task<D>) -> Result<(), Task<D>> {
        match self.inst_tx.send(Instruction::NewTask(task)) {
            Ok(_) => {
                let new_count = self.queue_size.fetch_add(1, Ordering::SeqCst);
                println!("sent (worker), {} -> {}", new_count - 1, new_count);
                Ok(())
            },
            Err(send_err) => {
                match send_err.0 {
                    Instruction::NewTask(inner) => Err(inner),
                    _ => panic!("terminate recieved in send_task SendError"),
                }
            }
        }
    }

    pub fn notify_compl(&self) {
        self.on_compl.1.notify_one();
    }

    pub fn read_count(&self) -> usize {
        self.queue_size.load(Ordering::SeqCst)
    }
}


pub fn pipe<D>() -> (PoolPipe<D>, WorkerPipe<D>) {
    let (inst_tx, inst_rx) = mpsc::channel();

    let inst_rx = Arc::new(Mutex::new(inst_rx));

    let queue_size = Arc::new(AtomicUsize::new(0));
    let on_compl = Arc::new((Mutex::new(()), Condvar::new()));

    let pool_pipe = PoolPipe {
        inst_tx: inst_tx.clone(),
        queue_size: queue_size.clone(),
        on_compl: on_compl.clone(),
    };

    let worker_pipe = WorkerPipe {
        inst_tx,
        inst_rx,
        queue_size,
        on_compl
    };

    (pool_pipe, worker_pipe)
}
