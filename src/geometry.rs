mod mesh;
mod sampling;
mod shapes;
mod surface;
mod types;

pub use mesh::Mesh;
pub use sampling::{sample_curve_at, sample_surface_at};
pub use types::{Edge, Object, Point, Vertex};