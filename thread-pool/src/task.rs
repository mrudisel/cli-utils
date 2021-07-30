/*
pub trait FnBox<R> {
    fn call(self: Box<Self>) -> R;
}


impl<R, F: FnOnce() -> R> FnBox<R> for F {
    fn call(self: Box<F>) -> R {
        (*self)()
    }
}
*/

pub type Task<D> = fn() -> Option<TaskResult<D>>;


#[derive(Clone, Debug)]
pub enum Shape<D> {
    Single(D),
    Batch(Vec<D>),
}

impl<D> From<D> for Shape<D> {
    fn from(data: D) -> Self {
        Shape::Single(data.into())
    }
}

impl<D> From<Vec<D>> for Shape<D> {
    fn from(data_batch: Vec<D>) -> Self {
        Shape::Batch(data_batch)
    }
}


#[derive(Default)]
pub struct TaskResult<D>
where
    D: Copy + Send + 'static
{
    results: Option<Vec<D>>,
    jobs: Option<Vec<Task<D>>>,
}


impl<D> TaskResult<D>
where
    D: Copy + Send + 'static
{
    pub fn new() -> Self {
        Self {results: Some(vec![]), jobs: Some(vec![])}
    }

    pub fn from(results: Option<Shape<D>>, jobs: Option<Shape<Task<D>>>) -> Self {
        let job_result = match results {
            Some(Shape::Single(result)) => Self::from_result(result),
            Some(Shape::Batch(results)) => Self::from_results(results),
            None => Self::new(),
        };

        match jobs {
            Some(Shape::Single(job)) => job_result.add_job(job),
            Some(Shape::Batch(jobs)) => job_result.add_jobs(jobs),
            None => job_result,
        }
    }

    pub fn from_result(result: D) -> Self {
        Self {results: Some(vec![result]), jobs: Some(vec![])}
    }

    pub fn from_results(results: Vec<D>) -> Self {
        Self {results: Some(results), jobs: Some(vec![])}
    }

    pub fn from_job(job: Task<D>) -> Self {
        Self {
            results: Some(vec![]),
            jobs: Some(vec![job])
        }
    }

    pub fn from_jobs(jobs: Vec<Task<D>>) -> Self {
        Self {
            results: Some(vec![]),
            jobs: Some(jobs)
        }
    }

    pub fn clear(mut self) -> Self {
        self.results.as_mut().expect("TaskResult already consumed").clear();
        self.jobs.as_mut().expect("TaskResult already consumed").clear();
        self
    }

    pub fn add_result(mut self, result: D) -> Self {
        self.results.as_mut().expect("TaskResult already consumed").push(result);
        self
    }

    pub fn add_results<L>(mut self, results: L) -> Self
    where
        L: Into<Vec<D>>
    {
        self.results.as_mut().expect("TaskResult already consumed").extend(results.into());
        self
    }

    pub fn add_job(mut self, job: Task<D>) -> Self {
        self.jobs.as_mut()
            .expect("TaskResult already consumed")
            .push(job);

        self
    }

    pub fn add_jobs(mut self, jobs: Vec<Task<D>>) -> Self {
        self.jobs.as_mut().expect("TaskResult already consumed").extend(jobs);
        self
    }

    pub fn get(mut self) -> (Option<Shape<D>>, Option<Shape<Task<D>>>) {
        let results = {
            let raw_results = self.results.take().expect("TaskResult already consumed");

            match raw_results.len() {
                0 => None,
                1 => Some(Shape::Single(raw_results[0])),
                _ => Some(Shape::Batch(raw_results))
            }
        };

        let jobs = {
            let mut raw_jobs = self.jobs.take().expect("TaskResult already consumed");
            match raw_jobs.len() {
                0 => None,
                1 => Some(Shape::Single(raw_jobs.remove(0))),
                _ => Some(Shape::Batch(raw_jobs)),
            }
        };

        (results, jobs)
    }

    pub fn is_empty(&self) -> bool {
        let num_results = self.results.as_ref().expect("TaskResult already consumed").len();
        let num_jobs = self.jobs.as_ref().expect("TaskResult already consumed").len();

        num_results == 0 && num_jobs == 0
    }

    pub fn to_opt(self) -> Option<Self> {
        if self.is_empty() {
            None
        }
        else {
            Some(self)
        }
    }

    pub fn has_results(&self) -> bool {
        ! self.results.as_ref().expect("").is_empty()
    }

    pub fn clone_results(&self) -> Option<Vec<D>> {
        self.results.clone()
    }

    pub fn clone_first_result(&self) -> Option<D> {
        self.results.as_ref().expect("").first().map(|first| first.clone().to_owned())
    }
}
