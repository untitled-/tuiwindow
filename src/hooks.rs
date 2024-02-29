use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct AsyncResource<T: Send> {
    task: Option<JoinHandle<()>>,
    channels: (Sender<T>, Receiver<T>),
    result: Option<T>,
}

impl<'a, T: Send + 'a> AsyncResource<T> {
    pub fn new() -> Self {
        Self {
            task: None,
            channels: mpsc::channel(),
            result: None,
        }
    }

    pub fn with_thread_spawning<F>(&mut self, task: F) -> &Option<T>
    where
        F: Send + 'static + FnOnce() -> T,
        T: Send + 'static,
    {
        let (tx, rx) = &self.channels;
        self.task.get_or_insert_with(|| {
            let thread_tx = tx.clone();
            thread::spawn(move || {
                thread_tx.send(task()).unwrap();
            })
        });

        if let Ok(res) = rx.try_recv() {
            self.result = Some(res)
        }
        &self.result
    }
}

impl<T: Send + 'static> Default for AsyncResource<T> {
    fn default() -> Self {
        Self::new()
    }
}
