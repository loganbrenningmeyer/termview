/**
 * 
 */
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Self {
        Self { x, y, width, height }
    }

    pub fn resize(
        &mut self, 
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    /// Split into top and bottom rectangles. Panics if the fraction is
    /// non-finite or outside 0..=1; endpoints allow an empty child.
    pub fn split_rows(&self, top_frac: f32) -> (Self, Self) {
        assert!(top_frac.is_finite() && (0.0..=1.0).contains(&top_frac),
            "row fraction must be finite and within 0..=1");

        let top_rows = ((self.height as f64 * f64::from(top_frac)).floor() as usize)
            .min(self.height);
        let bot_rows = self.height - top_rows;

        (
            Rect::new(
                self.x, self.y, 
                self.width, top_rows
            ),
            Rect::new(
                self.x, self.y + top_rows, 
                self.width, bot_rows
            ),
        )
    }

    /// Split into left and right rectangles. Panics if the fraction is
    /// non-finite or outside 0..=1; endpoints allow an empty child.
    pub fn split_cols(&self, left_frac: f32) -> (Self, Self) {
        assert!(left_frac.is_finite() && (0.0..=1.0).contains(&left_frac),
            "column fraction must be finite and within 0..=1");

        let top_cols = ((self.width as f64 * f64::from(left_frac)).floor() as usize)
            .min(self.width);
        let bot_cols = self.width - top_cols;

        (
            Rect::new(
                self.x, self.y, 
                top_cols, self.height
            ),
            Rect::new(
                self.x + top_cols, self.y, 
                bot_cols, self.height
            ),
        )
    }
}


/**
 * Recursive tree layout of split rules for pane Rects
 */
pub enum Layout {
    Leaf {
        pane_id: usize,
    },
    RowSplit {
        fraction: f32,  // portion assigned to the top child
        top: Box<Layout>,
        bottom: Box<Layout>,
    },
    ColumnSplit {
        fraction: f32,  // portion assigned to the left child
        left: Box<Layout>,
        right: Box<Layout>,
    },
}

impl Layout {
    /**
     * Recursively walk through the Layout tree and calculate the
     * rectangle for each pane using assign(pane_id, Rect)
     */
    pub fn visit_areas(
        &self,
        area: Rect,
        assign: &mut impl FnMut(usize, Rect),
    ) {
        match self {
            Self::Leaf { pane_id } => assign(*pane_id, area),

            Self::RowSplit { fraction, top, bottom } => {
                // Get top / bottom Rects
                let (top_area, bottom_area) = area.split_rows(*fraction);
                // Recursively visit sub-panes of top / bottom
                top.visit_areas(top_area, assign);
                bottom.visit_areas(bottom_area, assign);
            }

            Self::ColumnSplit { fraction, left, right } => {
                // Get left / right Rects
                let (left_area, right_area) = area.split_cols(*fraction);
                // Recursively visit sub-panes of left / right
                left.visit_areas(left_area, assign);
                right.visit_areas(right_area, assign);
            }
        }
    }

    /**
     * Recursively walk to find a leaf mutably by pane ID
     */
    pub fn find_leaf_mut(&mut self, target: usize) -> Option<&mut Layout> {
        match self {
            Self::Leaf { pane_id } => {
                if *pane_id == target {
                    Some(self)
                } else {
                    None
                }
            }

            Self::RowSplit { top, bottom, .. } => {
                top.find_leaf_mut(target)
                    .or_else(|| bottom.find_leaf_mut(target))
            }

            Self::ColumnSplit { left, right, .. } => {
                left.find_leaf_mut(target)
                    .or_else(|| right.find_leaf_mut(target))
            }
        }
    }

    /**
     * Remove leaf from the tree and replace its parent
     * split with the surviving sibling
     * - Returns surviving subtree's first pane id on success, None on failure
     */
    pub fn remove_leaf(&mut self, target: usize) -> Option<usize> {
        match self {
            Self::Leaf { .. } => None,

            Self::RowSplit { top: first, bottom: second, .. }
            | Self::ColumnSplit { left: first, right: second, .. } => {
                // Check whether either child is a Leaf with the target Pane ID
                // - If so, the sibling is the other
                let sibling = if matches!(
                    first.as_ref(),     // as_ref() because first is Box<>
                    Self::Leaf { pane_id } if *pane_id == target
                ) {
                    second
                } else if matches!(
                    second.as_ref(),
                    Self::Leaf { pane_id } if *pane_id == target
                ) {
                    first
                // If neither are the target, continue searching
                } else {
                    return first.remove_leaf(target)
                        .or_else(|| second.remove_leaf(target));
                };

                // Replace the parent split with the sibling
                // - mem::replace to create a temporary replacement value
                //   that lets Rust move the surviving sibling out
                let survivor = std::mem::replace(
                    sibling,
                    Box::new(Self::Leaf { pane_id: target }),   // target is temporary placeholder; doesn't matter
                );

                *self = *survivor;          // self is the current parent split calling the function
                Some(self.first_pane_id())  // surviving subtree's first pane id
            }
        }
    }

    /**
     * Get the first pane ID from the Layout tree root (top-left first)
     */
    pub fn first_pane_id(&self) -> usize {
        match self {
            Self::Leaf { pane_id } => *pane_id,
            Self::RowSplit { top, .. } => top.first_pane_id(),
            Self::ColumnSplit { left, .. } => left.first_pane_id(),
        }
    }
}
