use std::marker::PhantomData;

#[derive(Debug)]
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>, // We use Option<T> to represent empty slots
    head: usize,            // Index where the next element will be written
    tail: usize,            // Index of the oldest element
    capacity: usize,
    len: usize,
    _phantom: PhantomData<T>, // To tie T's lifetime
}

impl<T: Clone> RingBuffer<T> {
    /// Creates a new, empty `RingBuffer` with the specified capacity.
    ///
    /// Panics if capacity is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be greater than 0");

        // Initialize the buffer with `None` values.
        let buffer = vec![None; capacity];

        RingBuffer {
            buffer,
            head: 0,
            tail: 0,
            capacity,
            len: 0,
            _phantom: PhantomData,
        }
    }

    /// Returns the total capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of elements currently in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Adds an element to the buffer.
    /// If the buffer is full, the oldest element is overwritten.
    pub fn push(&mut self, item: T) {
        if self.is_full() {
            // Overwrite the oldest element (at tail)
            // We need to replace the Option<T> at the tail index
            self.buffer[self.head] = Some(item);
            // Move the tail forward
            self.tail = (self.tail + 1) % self.capacity;
        } else {
            // Add the element at the head
            self.buffer[self.head] = Some(item);
            self.len += 1;
        }

        // Move the head forward
        self.head = (self.head + 1) % self.capacity;
    }

    pub fn try_push(&mut self, item: T) {
        if self.is_full() {
            return;
        } else {
            // Add the element at the head
            self.buffer[self.head] = Some(item);
            self.len += 1;
        }

        // Move the head forward
        self.head = (self.head + 1) % self.capacity;
    }

    /// Removes and returns the oldest element from the buffer.
    /// Returns `None` if the buffer is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            // Take the element at the tail
            let item = self.buffer[self.tail].take(); // `take()` replaces the Some(T) with None

            // Move the tail forward
            self.tail = (self.tail + 1) % self.capacity;
            self.len -= 1;

            item
        }
    }

    /// Returns a reference to the oldest element in the buffer without removing it.
    /// Returns `None` if the buffer is empty.
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.buffer[self.tail].as_ref()
        }
    }

    /// Returns a mutable reference to the oldest element in the buffer without removing it.
    /// Returns `None` if the buffer is empty.
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.buffer[self.tail].as_mut()
        }
    }

    /// Removes all elements from the buffer.
    pub fn clear(&mut self) {
        // Reset head, tail, and length
        self.head = 0;
        self.tail = 0;
        self.len = 0;
        // Overwrite all elements with None to drop the contained values
        for i in 0..self.capacity {
            self.buffer[i] = None;
        }
    }
}
