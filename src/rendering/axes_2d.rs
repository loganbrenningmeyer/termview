use super::{
    Buffer, 
    Cell,
    Color,
    PlotLayout2d, 
    PlotArea2d, 
    PlotViewport2d,
    draw_text_in_area,
};


/**
 * 
 */
pub fn draw_axes_ticks(
    buffer: &mut Buffer,
    viewport: &PlotViewport2d,
    layout: &PlotLayout2d,
    outer: PlotArea2d,
    tick_x: Cell,
    tick_y: Cell,
    num_ticks: usize,
    color: Color,
) {
    if num_ticks < 2 {
        return;
    }

    // Define plotting area for datapoints
    let data_area = layout.data;
    let x_range = viewport.x_max - viewport.x_min;
    let y_range = viewport.y_max - viewport.y_min;

    // X-ticks: aligned with the data columns, label below axis
    for i in 0..num_ticks {
        let t = i as f64 / (num_ticks - 1) as f64;

        let x_col = data_area.left 
            + (t * data_area.width() as f64).round() as isize;

        let value = viewport.x_min + t * x_range;
        let label = format!("{value:.2}");
        let label_width = label.chars().count() as isize;

        buffer.set(x_col, layout.x_axis_y, tick_x);

        // Center the label beneath its tick
        let max_label_x = 
            (data_area.right - label_width + 1).max(data_area.left);
        let label_x = 
            (x_col - label_width / 2).clamp(data_area.left, max_label_x);

        draw_text_in_area(
            buffer, 
            outer, 
            label_x, 
            layout.x_axis_y + 1, 
            &label, 
            color,
        );
    }

    // Y-ticks: aligned with data rows, labels left of axis
    for i in 0..num_ticks {
        let t = i as f64 / (num_ticks - 1) as f64;

        let y_row = data_area.top 
            + (t * data_area.height() as f64).round() as isize;

        let value = viewport.y_max - t * y_range;
        let label = format!("{value:.2}");
        let label_width = label.chars().count() as isize;

        buffer.set(layout.y_axis_x, y_row, tick_y);

        // Right-align labels with one blank column before the axis
        let label_x = layout.y_axis_x - label_width - 1;

        draw_text_in_area(
            buffer,
            outer,
            label_x,
            y_row,
            &label,
            color,
        );
    }
}

/**
 * Draw 2D axes on the left and bottom 
 */
pub fn draw_axes_2d(
    buffer: &mut Buffer,
    layout: &PlotLayout2d,
    cell_x: Cell,
    cell_y: Cell,
) {
    // Horizontal axis, including the bottom-left corner.
    for x in layout.y_axis_x..=layout.data.right {
        buffer.set(x, layout.x_axis_y, cell_x);
    }

    // Vertical axis.
    for y in layout.data.top..=layout.x_axis_y {
        buffer.set(layout.y_axis_x, y, cell_y);
    }

    buffer.set(
        layout.y_axis_x,
        layout.x_axis_y,
        Cell::new('└').with_fg(cell_x.fg),
    );
}