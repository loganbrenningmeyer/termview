/**
 * Range of N floats between min and max
 */
pub fn rangef(min: f64, max: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            min + (i as f64) * 
                (max - min) / 
                ((n - 1) as f64)
        })
        .collect()
}