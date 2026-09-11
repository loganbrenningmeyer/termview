use std::collections::HashMap;

use crate::{
    geometry::{Mesh, Point}, 
    math::rangef, 
    parsing::TokenNode,
};


/**
 * Sample 2D curve over x-values given some time (t), 
 * mutating a vector of points (x, y)
 */
pub fn sample_curve_at(
    expression: &TokenNode,
    x_min: f64,
    x_max: f64,
    samples: usize,
    t: f64,
    points: &mut Vec<Point>,
) {
    // Initialize HashMap for AST expression evaluation
    let mut vars = HashMap::from([
        ("x".to_string(), 0.0),
        ("t".to_string(), t),
    ]);

    points.clear();

    // Split x range into num samples
    for x in rangef(x_min, x_max, samples, true) {
        *vars.get_mut("x").unwrap() = x;
        let y = expression.evaluate(&vars);

        if y.is_finite() {
            points.push(Point::new(x, y));
        }
    }
}


/**
 * Sample 3D surface over x/y-values given some time (t),
 * mutating a vector of points (x, y, z)
 */
pub fn sample_surface_at(
    expression: &TokenNode,
    t: f64,
    mesh: &mut Mesh,
) {
    // Initialize HashMap for evaluation
    let mut vars = HashMap::from([
        ("x".to_string(), 0.0),
        ("y".to_string(), 0.0),
        ("t".to_string(), t),
    ]);

    for vertex in &mut mesh.vertices {
        *vars.get_mut("x").unwrap() = vertex.position.x;
        *vars.get_mut("y").unwrap() = vertex.position.y;

        vertex.position.z = expression.evaluate(&vars);

    }
}