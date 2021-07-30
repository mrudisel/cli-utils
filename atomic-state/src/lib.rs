use std::sync::RwLock;

pub struct AtomicState<S> {
    inner_state: RwLock<S>,
}


impl<S> AtomicState<S>
where
    S: Copy
{
    fn get(&self) -> S {
        match self.inner_state.read() {
            Ok(inner) => *inner,
            Err(poisoned) => *poisoned.into_inner()
        }
    }

    fn update(&mut self, new_state: S) {
        match self.inner_state.write() {
            Ok(mut write_lock) => {
                *write_lock = new_state;
            },
            Err(poisoned) => {
                *poisoned.into_inner() = new_state;
            },
        }
    }
}
