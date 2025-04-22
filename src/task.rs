use std::{
    pin::Pin,
    sync::{Arc, Mutex, mpsc},
    task::{Context, Poll},
};

use futures::task::{self, ArcWake};

pub(crate) struct Task {
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>,
}
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}
impl Task {
    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    pub(crate) fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }

    pub fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();

        task_future.poll(&mut cx);
    }
}

struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}
impl TaskFuture {
    fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        };
    }
}
