use std::{
    pin::Pin,
    sync::{Arc, mpsc},
    task::{Context, Poll},
    thread,
    time::{Duration, Instant},
};

use task::Task;

mod task;

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let future = Delay {
            when: Instant::now() + Duration::from_millis(10),
        };

        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> Self {
        let (sender, scheduled) = mpsc::channel();
        Self { scheduled, sender }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll()
        }
    }
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello, World!");
            Poll::Ready("done")
        } else {
            let waker = cx.waker().clone();
            let when = self.when;

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                waker.wake()
            });

            Poll::Pending
        }
    }
}
