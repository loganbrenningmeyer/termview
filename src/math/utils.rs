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