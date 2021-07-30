#[macro_export]
macro_rules! aquire {
    ($mutex:expr) => {
        match $mutex.lock() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        }
    };
}

#[macro_export]
macro_rules! aquire_write {
    ($rwlock:expr) => {
        match $rwlock.write() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        }
    };
}

#[macro_export]
macro_rules! aquire_read {
    ($rwlock:expr) => {
        match $rwlock.read() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        }
    };
}

#[macro_export]
macro_rules! try_aquire {
    ($mutex:expr) => {
        match $mutex.try_lock() {
            Ok(lock) => Ok(lock),
            Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => Err($mutex)
        }
    };
}
