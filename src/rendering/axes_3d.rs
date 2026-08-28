use super::{BrailleBuffer, Buffer, Camera, Cell, Color, PlotViewport3d, WireframeStyle};
use crate::{
    geometry::{Edge, Mesh, Object, Vertex}, 
    math::{Transform, Vec3}, 
    rendering::{draw_line, draw_text, WireframeRenderer},
};


#[derive(Clone, Copy)]
struct AxisSpec {
    direction: Vec3,
    min: f64,
    max: f64,
    label: char,
    style: WireframeStyle,
    tick_style: Cell,
    label_style: Cell,
}

#[derive(Debug, Clone, Copy)]
pub struct Axes3dStyle {
    pub x: WireframeStyle,
    pub y: WireframeStyle,
    pub z: WireframeStyle,
    pub tick: Cell,
    pub label: Cell,
}

pub struct Axes3dRenderer {
    pub show_ticks: bool,
    pub show_labels: bool,
    pub ticks_per_axis: usize,
    pub tick_radius: f64,
    pub label_gap: f64,
}

impl Default for Axes3dStyle {
    fn default() -> Self {
        Self {
            x: WireframeStyle {
                edge: Cell::new('·').with_fg(Color::Rgb(190, 90, 100)),
                vertex: Cell::new('•').with_fg(Color::Rgb(190, 90, 100)),
            },
            y: WireframeStyle {
                edge: Cell::new('·').with_fg(Color::Rgb(90, 180, 120)),
                vertex: Cell::new('•').with_fg(Color::Rgb(90, 180, 120)),
            },
            z: WireframeStyle {
                edge: Cell::new('·').with_fg(Color::Rgb(90, 130, 200)),
                vertex: Cell::new('•').with_fg(Color::Rgb(90, 130, 200)),
            },
            tick: Cell::new('.').with_fg(Color::Rgb(125, 125, 135)),
            label: Cell::new(' ').with_fg(Color::Rgb(205, 205, 215)),
        }
    }
}

impl Default for Axes3dRenderer {
    fn default() -> Self {
        Self {
            show_ticks: true,
            show_labels: true,
            ticks_per_axis: 3,
            tick_radius: 0.75,
            label_gap: 1.0,
        }
    }
}

impl Axes3dRenderer {
    /**
     * Using WireframeRenderer, plots each 3D axis as a Braille wireframe
     * with two vertices and one edge, using the appropriate axis style.
     */
    pub fn render(
        &self,
        wireframe: &WireframeRenderer,
        viewport: &PlotViewport3d,
        transform: Transform,
        camera: &Camera,
        style: Axes3dStyle,
        buffer: &mut Buffer,
    ) {
        let display_aspect = buffer.display_aspect();
        let mut braille = BrailleBuffer::new(buffer.width(), buffer.height());

        self.render_lines_braille(
            wireframe,
            viewport,
            transform,
            camera,
            style,
            display_aspect,
            &mut braille,
        );
        braille.composite(buffer);

        self.render_annotations(
            wireframe,
            viewport,
            transform,
            camera,
            style,
            buffer,
        );
    }

    fn axis_specs(viewport: &PlotViewport3d, style: Axes3dStyle) -> [AxisSpec; 3] {
        [
            AxisSpec {
                direction: Vec3::X,
                min: viewport.x_min,
                max: viewport.x_max,
                label: 'X',
                style: style.x,
                tick_style: style.tick,
                label_style: style.label,
            },
            AxisSpec {
                direction: Vec3::Y,
                min: viewport.y_min,
                max: viewport.y_max,
                label: 'Y',
                style: style.y,
                tick_style: style.tick,
                label_style: style.label,
            },
            AxisSpec {
                direction: Vec3::Z,
                min: viewport.z_min,
                max: viewport.z_max,
                label: 'Z',
                style: style.z,
                tick_style: style.tick,
                label_style: style.label,
            },
        ]
    }

    pub(crate) fn render_lines_braille(
        &self,
        wireframe: &WireframeRenderer,
        viewport: &PlotViewport3d,
        transform: Transform,
        camera: &Camera,
        style: Axes3dStyle,
        display_aspect: f64,
        braille: &mut BrailleBuffer,
    ) {
        for (index, axis) in Self::axis_specs(viewport, style).into_iter().enumerate() {
            let start = axis.direction * axis.min;
            let end = axis.direction * axis.max;
            let object = self.make_axis(start, end, transform);

            wireframe.render_braille_into(
                std::slice::from_ref(&object),
                camera,
                axis.style,
                display_aspect,
                index as u8 + 1,
                braille,
            );
        }
    }

    pub(crate) fn render_annotations(
        &self,
        wireframe: &WireframeRenderer,
        viewport: &PlotViewport3d,
        transform: Transform,
        camera: &Camera,
        style: Axes3dStyle,
        buffer: &mut Buffer,
    ) {
        let mut origin_label_drawn = false;

        for axis in Self::axis_specs(viewport, style) {
            self.render_axis_annotations(
                wireframe,
                axis,
                transform,
                camera,
                &mut origin_label_drawn,
                buffer,
            );
        }
    }

    fn render_axis_annotations(
        &self,
        wireframe: &WireframeRenderer,
        axis: AxisSpec,
        transform: Transform,
        camera: &Camera,
        origin_label_drawn: &mut bool,
        buffer: &mut Buffer,
    ) {
        if self.show_ticks || self.show_labels {
            self.render_ticks_labels(
                wireframe,
                axis,
                transform,
                camera,
                origin_label_drawn,
                buffer,
            );
        }

        if self.show_labels {
            self.render_axis_label(
                wireframe, 
                axis, 
                transform, 
                camera, 
                buffer,
            );
        }
    }

    /**
     * Creates an Object for the axis with the given bounds and the 
     * given transform applied
     */
    fn make_axis(
        &self,
        start: Vec3,
        end: Vec3,
        transform: Transform,
    ) -> Object {
        Object::new(
            Mesh {
                vertices: vec![
                    Vertex { position: start },
                    Vertex { position: end },
                ],
                edges: vec![Edge::new(0, 1)],
            },
            transform,
        )
    }

    /**
     * 
     */
    fn render_ticks_labels(
        &self,
        wireframe: &WireframeRenderer,
        axis: AxisSpec,
        transform: Transform,
        camera: &Camera,
        origin_label_drawn: &mut bool,
        buffer: &mut Buffer,
    ) {
        // Project start/end points and find perpendicular screen direction
        let Some((orth_x, orth_y)) = self.axis_orth_proj(
            wireframe, 
            axis, 
            transform, 
            camera, 
            buffer
        ) else {
            return;
        };

        // Draw line extending from each anchor point in perpendicular direction
        for value in Self::tick_values(
            axis.min, 
            axis.max, 
            self.ticks_per_axis,
        ) {
            // Tick value along axis is flat axis dir * value
            let anchor_3d = axis.direction * value;

            let Some(anchor) = wireframe.project_object_point(
                anchor_3d,
                transform,
                camera,
                buffer,
            ) else {
                continue;
            };

            // Draw tick lines
            if self.show_ticks {
                // Define tick endpoints centered along anchor in 
                // perpendicular direction, and with tick radius 
                let x0 = anchor.x - (orth_x * self.tick_radius).round() as isize;
                let y0 = anchor.y - (orth_y * self.tick_radius).round() as isize;
    
                let x1 = anchor.x + (orth_x * self.tick_radius).round() as isize;
                let y1 = anchor.y + (orth_y * self.tick_radius).round() as isize;
    
                draw_line(
                    buffer,
                    x0,
                    y0,
                    x1,
                    y1,
                    axis.tick_style,
                );
            }

            // Draw tick value as text label
            let is_origin = value.abs() < 1e-10;

            if self.show_labels && (!is_origin || !*origin_label_drawn) {
                let text = if is_origin {
                    "0".to_string()
                } else {
                    format!("{value:.2}")
                };

                let gap = self.tick_radius + self.label_gap;

                // Shift labels by label_gap further away than the tick lines
                let label_x = anchor.x + (orth_x * gap).round() as isize;
                let label_y = anchor.y + (orth_y * gap).round() as isize;

                // Center label
                let label_x_center = 
                    label_x - text.chars().count() as isize / 2;

                draw_text(
                    buffer,
                    label_x_center,
                    label_y,
                    &text,
                    axis.label_style,
                );

                if is_origin {
                    *origin_label_drawn = true;
                }
            }
        }
    }

    /**
     * Computes stepped values between [min, max] for axis,
     * returning vector of the values
     */
    fn tick_values(min: f64, max: f64, count: usize) -> Vec<f64> {
        if count < 2
            || !min.is_finite()
            || !max.is_finite()
            || min >= max
        {
            return Vec::new();
        }

        let step = (max - min) / (count - 1) as f64;

        (0..count)
            .map(|i| min + i as f64 * step)
            .collect()
    }

    /**
     * Draw axis label at end of axis (e.g., X, Y, Z)
     */
    fn render_axis_label(
        &self,
        wireframe: &WireframeRenderer,
        axis: AxisSpec,
        transform: Transform,
        camera: &Camera,
        buffer: &mut Buffer,
    ) {
        let start_3d = axis.direction * axis.min;
        let end_3d = axis.direction * axis.max;

        let (Some(start), Some(end)) = (
            wireframe.project_object_point(
                start_3d,
                transform,
                camera,
                buffer,
            ),
            wireframe.project_object_point(
                end_3d,
                transform,
                camera,
                buffer,
            ),
        ) else {
            return;
        };

        // Normalized screen-space direction toward the positive endpoint.
        let dx = (end.x - start.x) as f64;
        let dy = (end.y - start.y) as f64;

        // Normalize in physical screen proportions so the label gap looks
        // consistent despite terminal cells being taller than they are wide.
        let cell_aspect = buffer.cell_aspect();
        let dx_display = dx * cell_aspect;
        let length = dx_display.hypot(dy);

        if length <= f64::EPSILON {
            return;
        }

        let direction_x = dx / length;
        let direction_y = dy / length;

        let gap = self.label_gap.max(1.0);

        let mut label_x =
            end.x + (direction_x * gap).round() as isize;
        let mut label_y =
            end.y + (direction_y * gap).round() as isize;

        // If extending outward leaves the buffer, place the label inward.
        let max_x = buffer.width().saturating_sub(1) as isize;
        let max_y = buffer.height().saturating_sub(1) as isize;

        if !(0..=max_x).contains(&label_x)
            || !(0..=max_y).contains(&label_y)
        {
            label_x =
                end.x - (direction_x * gap).round() as isize;
            label_y =
                end.y - (direction_y * gap).round() as isize;
        }

        let label_cell = Cell::new(axis.label)
            .with_fg(axis.style.edge.fg);

        draw_text(
            buffer,
            label_x,
            label_y,
            &axis.label.to_string(),
            label_cell,
        );
    }

    /**
     * Determine perpendicular direction to axis in screen space,
     * allows for placing ticks perpendicular out of the axis in the render
     */
    fn axis_orth_proj(
        &self,
        wireframe: &WireframeRenderer,
        axis: AxisSpec,
        transform: Transform,
        camera: &Camera,
        buffer: &mut Buffer,
    ) -> Option<(f64, f64)> {
        let start_3d = axis.direction * axis.min;
        let end_3d = axis.direction * axis.max;

        let start = wireframe.project_object_point(
            start_3d,
            transform,
            camera,
            buffer,
        )?;

        let end = wireframe.project_object_point(
            end_3d,
            transform,
            camera,
            buffer,
        )?;

        let dx = (end.x - start.x) as f64;
        let dy = (end.y - start.y) as f64;

        // Find the perpendicular in physical screen proportions. A raw
        // row/column perpendicular looks skewed because cells are not square.
        let cell_aspect = buffer.cell_aspect();
        let dx_display = dx * cell_aspect;
        let length = dx_display.hypot(dy);

        if length <= f64::EPSILON {
            return None;
        }

        // Convert the physical-screen perpendicular back into cell offsets.
        let mut orth_x = (-dy / length) / cell_aspect;
        let mut orth_y = dx_display / length;

        if orth_y < 0.0 {
            orth_x = -orth_x;
            orth_y = -orth_y;
        }

        Some((orth_x, orth_y))
    }
}
