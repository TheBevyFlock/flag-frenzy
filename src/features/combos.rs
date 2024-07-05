use std::iter::FusedIterator;

/// An [`Iterator`] that yields combonations of size `k` from pool of size `n`.
///
/// Note than `k` cannot be greater than `n`, or self [`Self::new()`] will panic.
///
/// Originally based on https://stackoverflow.com/a/65244323, but modified to be an asynchronous
/// iterator.
pub struct Combos {
    pool_size: usize,
    output: Box<[usize]>,
    i: usize,
    in_inner_loop: bool,
    is_done: bool,
}

impl Combos {
    pub fn new(n: usize, k: usize) -> Self {
        assert!(
            n >= k,
            "Cannot sample a group ({k}) larger than the original ({n})."
        );

        let mut output = vec![0; k].into_boxed_slice();
        let i = k - 1;

        output[i] = n - 1;

        Self {
            pool_size: n,
            // `k` is stored as the length of `output`, since it should never change.
            output,
            i,
            in_inner_loop: false,
            is_done: false,
        }
    }
}

impl Iterator for Combos {
    type Item = Box<[usize]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        // This implementation panics if n == k, so just return the original sample.
        if self.pool_size == self.output.len() {
            self.is_done = true;
            return Some((0..self.pool_size).collect());
        }

        match self.in_inner_loop {
            false => {
                let mut tmp = self.output[self.i];

                while self.i > 0 {
                    self.i -= 1;
                    tmp -= 1;

                    self.output[self.i] = tmp;
                }

                self.in_inner_loop = true;

                return Some(self.output.clone());
            }
            true => {
                self.output[self.i] -= 1;

                if self.output[self.i] != self.i {
                    // Equivalent of a break, using single-depth recursion to access
                    // `inner_loop == false` code.
                    self.in_inner_loop = false;
                    return self.next();
                }

                self.i += 1;

                if self.i == self.output.len() {
                    // Equivalent of a return.
                    self.is_done = true;
                }

                return Some(self.output.clone());
            }
        }
    }
}

impl FusedIterator for Combos {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::ncr;

    #[test]
    fn size() {
        let combos: Box<_> = Combos::new(7, 2).collect();

        assert_eq!(combos.len(), ncr(7, 2) as usize);

        for combo in combos.into_iter() {
            assert_eq!(combo.len(), 2);
        }
    }

    #[test]
    fn small() {
        let mut combos = Combos::new(3, 2);
        let expected = [[1, 2], [0, 2], [0, 1]];

        for combo in expected {
            assert_eq!(*combos.next().unwrap(), combo);
        }

        assert!(combos.next().is_none());
    }

    #[test]
    fn n_equals_k() {
        let mut combos = Combos::new(7, 7);
        let expected = [0, 1, 2, 3, 4, 5, 6];

        assert_eq!(*combos.next().unwrap(), expected);
        assert!(combos.next().is_none());
    }

    #[test]
    #[should_panic]
    fn n_less_than_k() {
        // `n` cannot be less than `k`.
        Combos::new(2, 3);
    }
}
