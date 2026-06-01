//! Latency percentile computation.
//!
//! Uses linear interpolation between closest ranks (the method NumPy calls
//! "linear" and Excel calls PERCENTILE.INC). For a sorted sample of `n` values
//! and percentile `p` in `[0, 100]`:
//!
//! ```text
//! rank  = (p / 100) * (n - 1)        // a 0-based fractional index
//! lower = floor(rank), upper = ceil(rank)
//! result = x[lower] + (rank - lower) * (x[upper] - x[lower])
//! ```
//!
//! This is exact at the endpoints (p=0 -> min, p=100 -> max) and interpolates
//! linearly in between.

/// Computes the `p`-th percentile (0..=100) of `values`. Returns `None` for an
/// empty input. The input need not be pre-sorted.
pub fn percentile(values: &[u64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let mut sorted: Vec<u64> = values.to_vec();
    sorted.sort_unstable();

    let n = sorted.len();
    if n == 1 {
        return Some(sorted[0] as f64);
    }

    let clamped = p.clamp(0.0, 100.0);
    let rank = (clamped / 100.0) * ((n - 1) as f64);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let frac = rank - (lower as f64);

    let lo = sorted[lower] as f64;
    let hi = sorted[upper] as f64;
    Some(lo + frac * (hi - lo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_none() {
        assert!(percentile(&[], 50.0).is_none());
    }

    #[test]
    fn single_value_is_itself() {
        assert_eq!(percentile(&[42], 0.0), Some(42.0));
        assert_eq!(percentile(&[42], 99.0), Some(42.0));
    }

    #[test]
    fn median_of_four_interpolates() {
        // sorted [10,20,30,40]; rank = 0.5*3 = 1.5 -> 20 + 0.5*(30-20) = 25.0
        assert_eq!(percentile(&[10, 20, 30, 40], 50.0), Some(25.0));
    }

    #[test]
    fn endpoints_are_exact() {
        assert_eq!(percentile(&[1, 2, 3, 4, 5], 0.0), Some(1.0));
        assert_eq!(percentile(&[1, 2, 3, 4, 5], 100.0), Some(5.0));
        assert_eq!(percentile(&[1, 2, 3, 4, 5], 50.0), Some(3.0));
    }

    #[test]
    fn input_need_not_be_sorted() {
        assert_eq!(percentile(&[40, 10, 30, 20], 50.0), Some(25.0));
    }
}
