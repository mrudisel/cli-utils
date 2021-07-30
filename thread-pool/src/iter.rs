/*

use super::job::JobResult;
use super::pool::ThreadPool;
*/
/*
pub struct ThreadPoolIter<'a, D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Clone + Send + 'static
{
    pool: &'a mut ThreadPool<D, F>,
}


impl<'a, D, F> From<&'a mut ThreadPool<D, F>> for ThreadPoolIter<'a, D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Clone + Send + 'static
{
    fn from(pool: &'a mut ThreadPool<D, F>) -> ThreadPoolIter<D, F> {
        ThreadPoolIter {pool}
    }
}


impl<'a, D, F> Iterator for ThreadPoolIter<'a, D, F>
where
    D: Copy + Send + 'static,
    F: FnOnce() -> Option<JobResult<D, F>>,
    F: Clone + Send + 'static
{
    type Item = &'a D;

    fn next(&mut self) -> Option<Self::Item> {
        self.pool.next_result()
    }
}
*/
