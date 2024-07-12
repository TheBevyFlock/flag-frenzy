use thiserror::Error;

#[derive(Error, PartialEq, Debug)]
#[error("integer operation overflowed")]
pub struct IntegerOverflowError;

/// Calculates the number of combinations for a pool of size `n` and a combo size of `r`.
///
/// Note that this calculation has a chance to overflow with higher values, in which case it will
/// return an [`Err`].
///
/// This algorithm is based off of <https://stackoverflow.com/a/65563202>, but has been modified to
/// use a `for` loop and handle integer overflow.
pub fn ncr(n: u128, r: u128) -> Result<u128, IntegerOverflowError> {
    if r > n {
        return Ok(0);
    }

    let mut acc = 1;

    for val in 1..=r {
        // Equivalent of `acc * (n - val + 1) / val` but with checked operations.
        acc = n
            .checked_sub(val)
            .and_then(|x| x.checked_add(1))
            .and_then(|x| x.checked_mul(acc))
            .and_then(|x| x.checked_div(val))
            .ok_or(IntegerOverflowError)?;
    }

    Ok(acc)
}

pub fn estimate_combos(n: u128, max_k: u128) -> Result<u128, IntegerOverflowError> {
    let mut sum = 0;

    for k in 0..=max_k {
        let c = ncr(n, k)?;
        sum = c.saturating_add(sum);
    }

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal() {
        assert_eq!(ncr(19, 2), Ok(171));
        assert_eq!(ncr(25, 4), Ok(12_650));
        assert_eq!(ncr(100, 5), Ok(75_287_520));
    }

    #[test]
    fn overflow() {
        assert!(ncr(1000, 50).is_err());
    }

    #[test]
    fn n_equals_k() {
        // Samples the entire pool, only 1 combo.
        assert_eq!(ncr(100, 100), Ok(1));
    }

    #[test]
    fn n_less_than_k() {
        // Edge case where you sample more than the original pool size, returns 0.
        assert_eq!(ncr(3, 4), Ok(0));
    }
}
