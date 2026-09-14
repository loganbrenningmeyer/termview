/**
 * Range of N floats between min and max
 * - end: false to exclude endopint, true to include it
 *        start point is always included
 */
pub fn rangef(min: f64, max: f64, n: usize, end: bool) -> Vec<f64> {
    let denom = if end { n - 1 } else { n } as f64;

    (0..n)
        .map(|i| {
            min + (i as f64) / denom
                * (max - min)
        })
        .collect()
}

/**
 * Expand [min, max] so the data occupies `fill` of the range,
 * centered. Constant ranges get a small nonzero span.
 */
pub fn padded_range(min: f64, max: f64, fill: f64) -> Option<(f64, f64)> {
    let center = min * 0.5 + max * 0.5;

    let half = if min == max {
        (center.abs() * 0.05).max(0.5)
    } else {
        (max * 0.5 - min * 0.5) / fill
    };

    let (lower, upper) = (center - half, center + half);
    (lower.is_finite() && upper.is_finite() && lower < upper)
        .then_some((lower, upper))
}
