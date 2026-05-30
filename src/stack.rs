//! A generic stack that handles over- and underflow.

/// Represents a stack with a fixed size set by `N`.
///
/// Overflowing the stack will drop the bottom element.
/// This means that arbitrarily many elements can be pushed,
/// but only the last `N` elements can be popped back off.
/// Trying to pop from an empty stack will return `T::default()`.
#[derive(Debug, Clone)]
pub struct Stack<T, const N: usize> {
    values: [T; N],
    /// Used to index the stack array as `values[total_pushes % N]`.
    total_pushes: usize,
    /// How many elements can be popped before the stack is empty.
    /// Since we allow overfilling this can be less than `total_pushes`.
    depth: usize,
}

impl<T: Default + Clone + Copy, const N: usize> Default for Stack<T, N> {
    fn default() -> Self {
        Self {
            values: [T::default(); N],
            total_pushes: 0,
            depth: 0,
        }
    }
}

impl<T: Default + Clone + Copy, const N: usize> Stack<T, N> {
    pub fn len(&self) -> usize {
        self.depth
    }

    pub fn push(&mut self, value: T) {
        self.values[self.total_pushes % N] = value;
        self.depth = N.min(self.depth + 1);
        self.total_pushes += 1;
    }

    pub fn pop(&mut self) -> T {
        if self.depth == 0 {
            return T::default();
        }
        self.total_pushes = self.total_pushes.saturating_sub(1);
        self.depth = self.depth.saturating_sub(1);
        self.values[self.total_pushes % N]
    }

    pub fn peek(&self) -> T {
        if self.depth == 0 {
            return T::default();
        }
        self.values[(self.total_pushes - 1) % N]
    }

    pub fn peek_at(&self, depth: usize) -> T {
        if depth >= self.len() {
            return T::default();
        }
        self.values[(self.total_pushes - 1 - depth) % N]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{MAX_STACK, MachineWord};
    use crate::mw;

    #[test]
    fn overflow_wraps() {
        let mut stack = Stack::<MachineWord, MAX_STACK>::default();
        // Push five more elements than the stack can store.
        for i in 0..(MAX_STACK + 5) {
            stack.push(mw!(i));
        }
        for i in (5..(MAX_STACK + 5)).rev() {
            assert_eq!(
                mw!(i),
                stack.pop(),
                "expected newest elements to survive overflow"
            );
        }
    }

    #[test]
    fn underflow_returns_default() {
        let mut stack = Stack::<MachineWord, MAX_STACK>::default();
        stack.push(mw!(1));
        assert_eq!(mw!(1), stack.pop(), "should pop the pushed value");
        for _ in 0..(2 * MAX_STACK) {
            assert_eq!(
                MachineWord::default(),
                stack.pop(),
                "underflow should return default"
            );
        }
    }

    #[test]
    fn push_pop_peek() {
        let mut stack = Stack::<MachineWord, MAX_STACK>::default();
        stack.push(mw!(1));
        stack.push(mw!(2));
        assert_eq!(
            mw!(2),
            stack.peek(),
            "peek should return top element without removing it"
        );
        assert_eq!(
            mw!(1),
            stack.peek_at(1),
            "peek_at(1) should return second element"
        );
        assert_eq!(mw!(2), stack.pop(), "first pop should return top element");
        assert_eq!(mw!(1), stack.pop(), "second pop should return next element");
        assert_eq!(
            MachineWord::default(),
            stack.pop(),
            "pop from empty should return default"
        );
    }

    #[test]
    fn len() {
        let mut stack = Stack::<MachineWord, MAX_STACK>::default();
        assert_eq!(0, stack.len(), "new stack should have length 0");
        stack.push(MachineWord::default());
        assert_eq!(1, stack.len(), "after one push length should be 1");
        let _ = stack.peek();
        assert_eq!(1, stack.len(), "peek should not change length");
        let _ = stack.pop();
        assert_eq!(0, stack.len(), "after pop length should be 0");
        // Popping from empty does not change len.
        let _ = stack.pop();
        assert_eq!(
            0,
            stack.len(),
            "popping empty stack should not change length"
        );
        for _ in 0..MAX_STACK {
            stack.push(MachineWord::default());
        }
        assert_eq!(
            MAX_STACK,
            stack.len(),
            "full stack should have length MAX_STACK"
        );
        stack.push(MachineWord::default());
        // Overflowing does not increase above max length.
        assert_eq!(
            MAX_STACK,
            stack.len(),
            "overflow should not increase length beyond MAX_STACK"
        );
        let _ = stack.pop();
        assert_eq!(
            MAX_STACK - 1,
            stack.len(),
            "pop from full stack should decrease length by 1"
        );
    }
}
