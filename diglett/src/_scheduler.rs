use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct JobSchedule {
    interval: u64,
    time: Duration,
}

impl JobSchedule {
    fn new(interval: u64, time: Duration) -> Self {
        JobSchedule { interval, time }
    }

    fn reschedule(&mut self, now: Duration) {
        self.time = now.checked_add(Duration::new(self.interval, 0)).unwrap();
    }
}

pub trait GetPinnedFuture {
    fn get_pinned(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}

#[derive(Clone)]
pub struct Job {
    f: Arc<Mutex<dyn GetPinnedFuture + Send>>,
    schedule: JobSchedule,
    repeat: bool,
}

impl Job {
    fn new(f: Arc<Mutex<dyn GetPinnedFuture + Send>>, schedule: JobSchedule, repeat: bool) -> Self {
        Job {
            f,
            schedule,
            repeat,
        }
    }
}

#[derive(Clone)]
struct FutureWrapper<F, T>
where
    F: FnMut() -> T,
    T: Future,
{
    f: F,
}

impl<F, T> FutureWrapper<F, T>
where
    F: FnMut() -> T,
    T: Future,
{
    fn new(f: F) -> Self {
        FutureWrapper { f }
    }
}

impl<F, T> GetPinnedFuture for FutureWrapper<F, T>
where
    F: FnMut() -> T,
    T: Future<Output = ()> + Send + 'static,
{
    fn get_pinned(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin((self.f)())
    }
}

pub struct Scheduler {
    jobs: HashMap<Uuid, Job>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            jobs: HashMap::new(),
        }
    }

    pub fn schedule<F, T>(&mut self, f: F, interval: u64)
    where
        F: FnMut() -> T + Send + 'static,
        T: Future<Output = ()> + Send + 'static,
    {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        self.jobs.insert(
            Uuid::new_v4(),
            Job::new(
                Arc::new(Mutex::new(FutureWrapper::new(f))),
                JobSchedule::new(
                    interval,
                    now.checked_add(Duration::new(interval, 0)).unwrap(),
                ),
                true,
            ),
        );
    }

    pub fn schedule_once<F, T>(&mut self, f: F, interval: u64)
    where
        F: FnMut() -> T + Send + 'static,
        T: Future<Output = ()> + Send + 'static,
    {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        self.jobs.insert(
            Uuid::new_v4(),
            Job::new(
                Arc::new(Mutex::new(FutureWrapper::new(f))),
                JobSchedule::new(
                    interval,
                    now.checked_add(Duration::new(interval, 0)).unwrap(),
                ),
                false,
            ),
        );
    }

    pub async fn run_pending(&mut self) {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let mut futures = Vec::new();
        let mut remove_ids = Vec::new();
        let mut reschedule_jobs = Vec::new();
        for job in self.jobs.iter_mut() {
            if job.1.schedule.time > now {
                remove_ids.push(job.0.to_owned());
                futures.push(job.1.f.lock().await.get_pinned());

                if job.1.repeat {
                    reschedule_jobs.push(job.1.clone());
                }
            }
        }

        for id in remove_ids {
            self.jobs.remove(&id);
        }

        for mut job in reschedule_jobs.into_iter() {
            job.schedule.reschedule(now);
            self.jobs.insert(Uuid::new_v4(), job);
        }

        for f in futures {
            f.await
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::Scheduler;

    #[tokio::test]
    async fn test_scheduler() {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let mut scheduler = Scheduler::new();
        scheduler.schedule_once(|| async { println!("{} test", now.as_millis()) }, 2);

        tokio::spawn(async move {
            loop {
                scheduler.run_pending().await;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        tokio::time::sleep(Duration::from_millis(3000)).await;
        assert_eq!(true, false);
    }
}
