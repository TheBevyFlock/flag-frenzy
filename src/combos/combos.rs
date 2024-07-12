use std::iter::FusedIterator;

/// A fused [`Iterator`] that yields combinations ([`Box<[usize]>`](slice)).
///
/// ```no_run
/// // From a pool of size 3, find all length-2 combinations.
/// for combo in Combos::new(3, 2) {
///     println!("{combo:?}");
/// }
/// ```
///
/// The slice yielded by this iterator contains unique [`usize`]s in the range of `0..n`.
/// `combo.len()` will always equal `k`.
///
/// Thank you to <https://stackoverflow.com/a/65244323> for the original implementation. It has
/// since been modified to be an [`Iterator`] with a few more optimizations.
pub struct Combos {
    /// The pool size, also known as `n`.
    pool_size: usize,
    /// Is worked on in-place to calculate the next combination.
    ///
    /// The length of this slice is `k`. Any yielded combinations are cloned from this.
    output: Box<[usize]>,
    /// The index into the `output`.
    i: usize,
    /// Used to track whether the iterator is within the outer or inner loop, based on the original
    /// implementation.
    in_inner_loop: bool,
    /// Used to track whether this iterator has finished yielding all combinations.
    is_done: bool,
}

impl Combos {
    /// Returns a new [`Combos`] with a pool of size `n` that will yield combinations with length
    /// `k`.
    ///
    /// # Special casing
    ///
    /// If `k` is 0, this iterator will yield one empty array `[]` before yielding [`None`].
    ///
    /// # Panics
    ///
    /// If `n < k`.
    pub fn new(n: usize, k: usize) -> Self {
        assert!(
            n >= k,
            "Cannot sample a group ({k}) larger than the original ({n})."
        );

        // Edge case where it samples a combination size of 0.
        if k == 0 {
            return Self {
                pool_size: n,
                output: Box::new([]),
                i: 0,
                in_inner_loop: false,
                is_done: false,
            };
        }

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

        if self.output.len() == 0 {
            self.is_done = true;
            return Some(Box::new([]));
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

                Some(self.output.clone())
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

                Some(self.output.clone())
            }
        }
    }
}

impl FusedIterator for Combos {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combos::ncr;

    #[test]
    fn size() {
        let combos: Box<_> = Combos::new(7, 2).collect();

        assert_eq!(combos.len() as u128, ncr(7, 2).unwrap());

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

    #[test]
    fn k_is_zero() {
        let mut combos = Combos::new(2, 0);

        assert_eq!(*combos.next().unwrap(), [0; 0]);
    }
}
