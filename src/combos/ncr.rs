/// Calculates the number of combinations for a pool of size `n` and a combo size of `r`.
///
/// Note that this calculation is relatively slow and has a chance to overflow with larger numbers.
/// Prefer caching the output of this function instead of repeatedly calculating the result.
///
/// Thank you <https://stackoverflow.com/a/65563202>.
pub fn ncr(n: u64, r: u64) -> u64 {
    if r > n {
        0
    } else {
        (1..=r.min(n - r)).fold(1, |acc, val| acc * (n - val + 1) / val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal() {
        assert_eq!(ncr(19, 2), 171);
        assert_eq!(ncr(25, 4), 12_650);
    }

    #[test]
    fn n_equals_k() {
        // Samples the entire pool, only 1 combo.
        assert_eq!(ncr(100, 100), 1);
    }

    #[test]
    fn n_less_than_k() {
        // Edge case where you sample more than the original pool size, returns 0.
        assert_eq!(ncr(3, 4), 0);
    }
}
