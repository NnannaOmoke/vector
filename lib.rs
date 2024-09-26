use core::fmt;

use std::alloc::alloc;
use std::alloc::realloc;
use std::alloc::Layout;

use std::ptr::drop_in_place;
use std::ptr::slice_from_raw_parts_mut;
// use std::ptr::NonNull;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;

use std::iter::DoubleEndedIterator;
use std::iter::IntoIterator;
use std::iter::Iterator;

use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;

use std::fmt::Debug;
use std::fmt::Display;

use std::cmp::Ordering;

#[derive(Hash)]
pub struct Vector<T> {
    head: *mut T,
    capacity: usize,
    length: usize,
}

impl<T> Vector<T> {
    unsafe fn __new() -> Self {
        let ptr = alloc(Layout::new::<T>()) as *mut T;
        Self {
            head: ptr,
            capacity: 0,
            length: 0,
        }
    }

    unsafe fn __increment_cap(&mut self) {
        if self.capacity == 0 {
            self.capacity = 1;
        } else {
            self.capacity = self.capacity * 2;
        }
        //reallocate memory?
        let new_ptr = realloc(self.head as *mut u8, Layout::new::<T>(), self.capacity);
        self.head = new_ptr as *mut T;
    }

    unsafe fn __decrement_cap(&mut self) {}

    unsafe fn __raw_push(&mut self, elem: T) {
        if self.length == self.capacity {
            self.__increment_cap();
        }
        self.head.add(self.length).write(elem);
        self.length += 1;
    }

    unsafe fn __raw_pop(&mut self) -> T {
        let value = self.head.add(self.length - 1).read();
        self.head.add(self.length - 1).drop_in_place();
        self.length -= 1;
        value
    }

    unsafe fn __delete_inplace(&mut self, index: usize) {
        if index > self.length - 1 {
            return;
        }
        // self.head.add(index).drop_in_place();
        for current in index + 1..self.length {
            let val = self.head.add(current).read();
            self.head.add(current - 1).write(val);
        }
        self.length -= 1;
    }

    unsafe fn __replace(&mut self, index: usize, elem: T) {
        if index > self.length - 1 {
            return;
        }
        self.head.add(index).write(elem);
    }

    unsafe fn __get(&self, index: usize) -> T {
        self.head.add(index).read()
    }

    //convenience functions
    fn get_result(&self, index: usize) -> Option<T> {
        if index > self.length - 1 {
            None
        } else {
            unsafe { Some(self.__get(index)) }
        }
    }

    fn get_as_ref(&self, index: usize) -> &T {
        if index > self.length - 1 {
            panic! {"Index out of bounds"}
        } else {
            unsafe { self.head.add(index).as_ref().unwrap() }
        }
    }

    fn get_as_ref_mut(&mut self, index: usize) -> &mut T {
        if index > self.length - 1 {
            panic!("Index out of bounds")
        } else {
            unsafe { self.head.add(index).as_mut().unwrap() }
        }
    }

    fn get_range(&self, range: Range<usize>) -> &[T] {
        if range.start > self.length - 1 || range.end > self.length - 1 {
            panic!("indice(s) out of bounds")
        } else {
            unsafe { from_raw_parts(self.head.add(range.start), range.end - range.start) }
        }
    }

    fn get_range_mut(&mut self, range: Range<usize>) -> &mut [T] {
        if range.start > self.length - 1 || range.end > self.length - 1 {
            panic!("indice(s) out of bounds")
        } else {
            unsafe { from_raw_parts_mut(self.head.add(range.start), range.end - range.start) }
        }
    }

    //the public safe API
    pub fn new() -> Self {
        unsafe { Self::__new() }
    }

    pub fn push(&mut self, elem: T) {
        unsafe { self.__raw_push(elem) }
    }

    pub fn pop(&mut self) -> T {
        unsafe { self.__raw_pop() }
    }

    pub fn delete_inplace(&mut self, index: usize) {
        unsafe { self.__delete_inplace(index) }
    }

    pub fn replace(&mut self, index: usize, elem: T) {
        unsafe { self.__replace(index, elem) }
    }

    pub fn get(&self, index: usize) -> T {
        if index > self.length - 1 {
            panic!("Invalid index")
        }
        unsafe { self.__get(index) }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.head as *const T, self.length) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.head, self.length) }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    //public unsafe operations
    //can segfault!
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        self.__get(index)
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = VecIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            internal: self,
            state: 0,
        }
    }
}

pub struct VecIterator<T> {
    internal: Vector<T>,
    state: usize,
}

impl<T> Iterator for VecIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.state += 1;
        self.internal.get_result(self.state)
    }
}

impl<T> DoubleEndedIterator for VecIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.state += 1;
        self.internal
            .get_result((self.internal.len() - 1) - self.state)
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        unsafe {
            //totally copied from Vec<T>
            //use it to clean up heap allocations and thus prevent memory leaks
            drop_in_place(slice_from_raw_parts_mut(self.head, self.len()))
        }
    }
}

impl<T> Index<usize> for Vector<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.get_as_ref(index)
    }
}

impl<T> Index<Range<usize>> for Vector<T> {
    type Output = [T];
    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        self.get_range(range)
    }
}

impl<T> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_as_ref_mut(index)
    }
}

impl<T> IndexMut<Range<usize>> for Vector<T> {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        self.get_range_mut(index)
    }
}

impl<T: Debug> Debug for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let intermediate = self.as_slice();
        fmt::Debug::fmt(intermediate, f)
    }
}

impl<T: Debug> Display for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let intermediate = self.as_slice();
        write!(f, "{:?}", intermediate)
    }
}

impl<T: PartialEq> PartialEq for Vector<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialOrd> PartialOrd for Vector<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for Vector<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialOrd> PartialOrd<Vec<T>> for Vector<T> {
    fn partial_cmp(&self, other: &Vec<T>) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: PartialEq> PartialEq<[T]> for Vector<T> {
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialOrd> PartialOrd<[T]> for Vector<T> {
    fn partial_cmp(&self, other: &[T]) -> Option<Ordering> {
        self.as_slice().partial_cmp(other)
    }
}
//THIS CAN, AND WILL BREAK. IT IS UB. DO NOT FUCKING USE THIS UNLESS YOU HAVE BALLS OF STEEL
//THIS IS STUPIDLY UNSAFE AND YOU SHOULD NEVER DO THIS, EVER, BUT IT'S THERE JUST BECAUSE
impl<'lt, T: Clone> From<&'lt [T]> for Vector<T> {
    fn from(other: &'lt [T]) -> Self {
        let len = other.len();
        Self {
            //what we're doing is setting the capacity to the same as the length
            //so anytime we want to add, we actually reallocate and avoid any issues we may get in the future
            head: other.as_ptr() as *mut T,
            capacity: len,
            length: len,
        }
    }
}

#[macro_export]
macro_rules! vector {
    ($($lit: literal), *) => {
        {
            let mut vector = Vector::new();
            $(vector.push($lit);)*
            vector
        }
    };

    ($($eval: expr), *) => {
        {
            let mut vector = Vector::new();
            $(vector.push($eval);)*
            vector
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functions() {
        let mut vector = Vector::new();
        vector.push(76);
        vector.push(77);
        assert_eq!(vector.len(), 2);
        assert_eq!(vector.pop(), 77);
        assert_eq!(vector.len(), 1);
        for i in 0..3 {
            vector.push(i);
        }
        vector.delete_inplace(0);
        assert_eq!(vector, vec![76, 1, 2]);
    }

    #[test]
    fn test_macros() {
        let vector = vector! {1, 2, 3, 4, 5};
        let vector_two = vector! {1 + 0, {let val = 1 + 4; val}, 18/3};
        assert_eq!(vector, vec![1, 2, 3, 4, 5]);
        assert_eq!(
            vector_two,
            vec![
                1 + 0,
                {
                    let val = 1 + 4;
                    val
                },
                18 / 3
            ]
        )
    }
}
