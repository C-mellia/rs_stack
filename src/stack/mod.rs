use std::alloc::{alloc, dealloc, Layout};
use std::mem::size_of;
use std::ops::{Index, IndexMut};
use std::ptr::null;

#[derive(Debug)]
pub struct Stack<T>
where
    T: Sized + Copy,
{
    layout: Layout,
    mem: *mut T,
    top: *mut T,
}

fn cap_inc(mut cap: usize, supposed_size: usize) -> usize {
    while cap < supposed_size {
        cap = if cap != 0 { cap * 2 } else { 10 };
    }
    cap
}

impl<T> Stack<T>
where
    T: Sized + Copy,
{
    pub fn new(size: usize) -> Self {
        let cap = cap_inc(0, size);
        let layout = Layout::array::<T>(cap).expect("Arithmetic overflow");
        unsafe {
            let mem = if cap != 0 {
                alloc(layout) as *mut T
            } else {
                null::<T>() as *mut T
            };
            let top = if cap != 0 {
                mem.add(cap / size_of::<T>())
            } else {
                mem
            };
            Stack { layout, mem, top }
        }
    }

    pub fn size(&self) -> usize {
        unsafe {
            self.mem
                .add(self.layout.size() / size_of::<T>())
                .offset_from(self.top) as usize
        }
    }

    pub fn cap(&self) -> usize {
        self.layout.size()
    }

    pub fn full(&self) -> bool {
        self.top == self.mem
    }

    pub fn empty(&self) -> bool {
        unsafe { self.top == self.mem.add(self.layout.size() / size_of::<T>()) }
    }

    fn check_cap(&mut self, supposed_size: usize) {
        let cap = self.layout.size();
        let new_cap = cap_inc(cap, supposed_size);
        if new_cap > cap {
            self.realloc(new_cap);
        }
    }

    fn realloc(&mut self, new_cap: usize) {
        unsafe {
            let layout = Layout::array::<T>(new_cap).expect("Arithmetic overflow");
            let new_mem = alloc(layout) as *mut T;
            self.top = rmemcpy(
                new_mem.add(new_cap),
                self.top,
                self.mem.add(self.layout.size() / size_of::<T>()),
            );
            dealloc(self.mem as *mut u8, self.layout);
            self.layout = layout;
            self.mem = new_mem;
        }
    }

    pub fn push(&mut self, val: T) {
        self.check_cap(self.size() + 1);
        unsafe {
            self.top = self.top.sub(1);
            self.top.write(val);
        }
    }

    pub fn push_slice(&mut self, slice: &[T]) {
        self.check_cap(self.size() + slice.len());
        unsafe {
            let ptr = slice.as_ptr() as *mut T;
            self.top = rmemcpy(self.top, ptr as *mut T, ptr.add(slice.len()));
        }
    }

    pub fn top(&self) -> Option<T> {
        if self.size() == 0 {
            None
        } else {
            unsafe { Some(self.top.read()) }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size() == 0 {
            None
        } else {
            unsafe {
                let val = self.top.read();
                self.top = self.top.add(1);
                Some(val)
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe { self.top = self.mem.add(self.layout.size() / size_of::<T>()) }
    }
}

impl<T> Stack<T>
where
    T: Sized + Copy,
{
    pub fn iter(&self) -> StackIter<T> {
        StackIter::new(self)
    }
}

fn rmemcpy<T>(mut dst: *mut T, src: *mut T, mut end: *mut T) -> *mut T
where
    T: Sized + Copy,
{
    unsafe {
        while src < end {
            end = end.sub(1);
            dst = dst.sub(1);
            dst.write(end.read());
        }
        dst
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackIter<'a, T> {
    slice: &'a [T],
    index: usize,
}

impl<'a, T> StackIter<'a, T>
where
    T: Sized + Copy,
{
    pub fn new(stack: &Stack<T>) -> Self {
        StackIter {
            slice: unsafe { std::slice::from_raw_parts(stack.top, stack.size()) },
            index: 0,
        }
    }
}

impl<'a, T> Iterator for StackIter<'a, T>
where
    T: Sized + Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let item = &self.slice[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T> Default for Stack<T>
where
    T: Sized + Copy,
{
    fn default() -> Self {
        Stack {
            layout: Layout::array::<T>(0).expect("Arithmetic overflow"),
            mem: null::<T>() as *mut T,
            top: null::<T>() as *mut T,
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Stack<T>
where
    T: Sized + Copy,
{
    fn from(arr: [T; N]) -> Self {
        let mut stack = Stack::default();
        for item in arr.iter().rev() {
            stack.push(*item);
        }
        stack
    }
}

impl<T> From<Vec<T>> for Stack<T>
where
    T: Sized + Copy,
{
    fn from(vec: Vec<T>) -> Self {
        let mut stack = Stack::default();
        for item in vec.iter().rev() {
            stack.push(*item);
        }
        stack
    }
}

impl<T> FromIterator<T> for Stack<T>
where
    T: Sized + Copy,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut stack = Stack::default();
        for item in iter {
            stack.push(item);
        }
        stack
    }
}

impl<T> Clone for Stack<T>
where
    T: Sized + Copy,
{
    fn clone(&self) -> Self {
        unsafe {
            let cap = cap_inc(0, self.size());
            let layout = Layout::array::<T>(cap).expect("Arithmetic overflow");
            let mem = alloc(layout) as *mut T;
            let top = rmemcpy(
                mem.add(cap),
                self.top,
                self.mem.add(self.layout.size() / size_of::<T>()),
            );
            Stack { layout, mem, top }
        }
    }
}

impl<T> Drop for Stack<T>
where
    T: Sized + Copy,
{
    fn drop(&mut self) {
        unsafe {
            dealloc(self.mem as *mut u8, self.layout);
        }
    }
}

impl<T> IndexMut<usize> for Stack<T>
where
    T: Sized + Copy,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.mem.add(index) }
    }
}

impl<T> Index<usize> for Stack<T>
where
    T: Sized + Copy,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.top.add(index) }
    }
}
