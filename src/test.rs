#[cfg(test)]
mod tests {
    use std::{
        alloc::{dealloc, Layout},
        ptr::null,
    };

    use project23::stack::Stack;

    #[test]
    fn null_dealloc() {
        // make sure deallocating null pointer is ok
        let layout = Layout::array::<i32>(0).expect("Arithmetic overflow");
        unsafe {
            dealloc(null::<i32>() as *mut u8, layout);
        }
    }

    #[test]
    fn zero_sized_stack() {
        let _ = Stack::<i32>::new(0);
    }

    #[test]
    fn reallocating() {
        let mut stack = Stack::<i32>::default();
        for i in 0..10 {
            stack.push(i);
        }
        for (ind, item) in stack.iter().enumerate() {
            assert_eq!(*item, 9 - ind as i32)
        }
    }

    #[test]
    fn clearing_stack() {
        let mut stack: Stack<_> = [1, 2, 3, 4, 5].into();
        stack.clear();
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn pushing_slice_in_order() {
        let mut stack = Stack::<i32>::default();
        stack.push_slice(&[1, 2, 3, 4, 5]);
        for (ind, item) in stack.iter().enumerate() {
            assert_eq!(*item, 1 + ind as i32)
        }
    }
}
