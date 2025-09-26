use std::{
    ops::{Deref, DerefMut},
    thread::ThreadId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ThreadCell<T> {
    data: T,
    write_thread: Option<ThreadId>,
}

impl<T> ThreadCell<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data,
            write_thread: None,
        }
    }

    /// Reset the write thread id.
    /// Allows for multiple threads write data, under user defined conditions.
    /// ```
    /// std::thread::spawn(|| {
    ///     //Write some data.
    /// }).join().unwrap();
    ///
    /// //data.reset()
    ///
    /// std::thread::spawn(|| {
    ///     loop {
    ///         //Write some more data.
    ///     }
    /// });
    /// ```
    pub const unsafe fn reset_thread(&mut self) {
        self.write_thread = None
    }
}

impl<T> Deref for ThreadCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        //We don't care how many threads read the data.
        &self.data
    }
}

//Do not allow for double writes.
impl<T> DerefMut for ThreadCell<T> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let id = std::thread::current().id();
        if let Some(write_thread) = self.write_thread {
            if id != write_thread {
                panic!(
                    "Tried to write data from {:?} but it has already been mutability accessed from {:?}.",
                    id, write_thread
                );
            }
        } else {
            self.write_thread = Some(id);
        }

        &mut self.data
    }
}
