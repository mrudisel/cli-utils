
use std::convert::TryFrom;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;

use std::thread::{self, JoinHandle};

use crate::pipe::WorkerPipe;
use crate::task::{Instruction, TaskHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    Idle = 0,
    Busy = 1,
    Terminated = 2,
    Panicked = 3,
}


/// ```
/// use std::convert::TryFrom;
/// use pool::worker::WorkerState;
///
/// let idle_usize: usize = 0;
/// let busy_usize: usize = 1;
/// let term_usize: usize = 2;
/// let panic_usize: usize = 3;
///
/// let bad_usizes: [usize; 5] = [4, 10, 100, 42, 7];
///
/// assert_eq!(WorkerState::try_from(idle_usize).unwrap(), WorkerState::Idle);
/// assert_eq!(WorkerState::try_from(busy_usize).unwrap(), WorkerState::Busy);
/// assert_eq!(WorkerState::try_from(term_usize).unwrap(), WorkerState::Terminated);
/// assert_eq!(WorkerState::try_from(panic_usize).unwrap(), WorkerState::Panicked);
///
/// for bad_usize in bad_usizes {
///     assert_eq!(WorkerState::try_from(bad_usize).unwrap_err(), bad_usize);
/// }
/// ```
impl TryFrom<usize> for WorkerState {
    type Error = usize;

    fn try_from(n: usize) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(WorkerState::Idle),
            1 => Ok(WorkerState::Busy),
            2 => Ok(WorkerState::Terminated),
            3 => Ok(WorkerState::Panicked),
            _ => Err(n)
        }
    }
}

#[derive(Debug)]
pub struct Worker {
    pub id: usize,

    control: Weak<()>,

    /// WorkerState, encoded as an Arc<AtomicUsize> for built in
    /// thread saftey and atomic operations. Call Worker::state to
    /// get the state as a full WorkerState enum.
    state: Arc<AtomicUsize>,

    handle: JoinHandle<()>,

    // term_signaled: Arc<AtomicBool>
}


impl Worker {
    pub fn new<D>(id: usize, pipe: Arc<WorkerPipe<D>>, result_tx: Sender<D>) -> Self
    where
        D: Send + 'static
    {
        let thread_return = Arc::new(());
        let control = Arc::downgrade(&thread_return);

        let state = Arc::new(AtomicUsize::new(0));
        let state_clone = state.clone();

        let handle = thread::spawn(move || {
            loop {
                let inst = pipe.recv_inst().expect("pool sender has disconnected");

                state_clone.store(WorkerState::Busy as usize, Ordering::SeqCst);

                match inst {
                    Instruction::NewTask(task) => task(TaskHandler::new(&pipe, &result_tx)),
                    Instruction::Terminate => break,
                }

                state_clone.store(WorkerState::Idle as usize, Ordering::SeqCst);
                pipe.notify_compl();
            }

            state_clone.store(WorkerState::Terminated as usize, Ordering::SeqCst);

            *thread_return
        });

        Self {id, control, handle, state}
    }

    pub fn join(self) -> Result<(), Box<dyn std::any::Any + Send>> {
        self.handle.join()
    }

    pub fn alive(&self) -> bool {
        self.control.upgrade().is_some()
    }

    pub fn state(&self) -> WorkerState {
        let curr_state = WorkerState::try_from(self.state.load(Ordering::SeqCst))
            .unwrap_or_else(|n| panic!("Cannot cast {} to a valid WorkerState", n));

        // If the thread is still running, just return the current state.
        // If not, check if the thread terminated properly
        match self.control.upgrade() {
            Some(_) => curr_state,
            None => {
                if curr_state == WorkerState::Terminated {
                    WorkerState::Terminated
                }
                else {
                    WorkerState::Panicked
                }
            }
        }
    }
}
