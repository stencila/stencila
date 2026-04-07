/// Axis-aligned bounding box in viewBox coordinates.
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BBox {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if this box overlaps with another (non-zero intersection area).
    #[must_use]
    pub fn intersects(&self, other: &BBox) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns true if this box is fully contained within `outer`.
    #[must_use]
    pub fn within(&self, outer: &BBox) -> bool {
        self.x >= outer.x
            && self.y >= outer.y
            && self.x + self.width <= outer.x + outer.width
            && self.y + self.height <= outer.y + outer.height
    }

    /// Returns true if a line segment from (x1,y1) to (x2,y2) intersects this box.
    ///
    /// Uses the Liang-Barsky algorithm for line-rectangle clipping.
    #[must_use]
    pub fn intersects_line(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let right = self.x + self.width;
        let bottom = self.y + self.height;

        // Parametric clipping: find t range [t0, t1] where line is inside rect
        let mut t0: f64 = 0.0;
        let mut t1: f64 = 1.0;

        let clip = |p: f64, q: f64, t0: &mut f64, t1: &mut f64| -> bool {
            if p.abs() < f64::EPSILON {
                // Line is parallel to this edge
                return q >= 0.0;
            }
            let t = q / p;
            if p < 0.0 {
                // Entering
                if t > *t1 {
                    return false;
                }
                if t > *t0 {
                    *t0 = t;
                }
            } else {
                // Leaving
                if t < *t0 {
                    return false;
                }
                if t < *t1 {
                    *t1 = t;
                }
            }
            true
        };

        clip(-dx, x1 - self.x, &mut t0, &mut t1)
            && clip(dx, right - x1, &mut t0, &mut t1)
            && clip(-dy, y1 - self.y, &mut t0, &mut t1)
            && clip(dy, bottom - y1, &mut t0, &mut t1)
    }

    /// Returns true if a point (x, y) is inside this box.
    #[must_use]
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_overlap() {
        let a = BBox::new(0.0, 0.0, 10.0, 10.0);
        let b = BBox::new(20.0, 20.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn overlap() {
        let a = BBox::new(0.0, 0.0, 10.0, 10.0);
        let b = BBox::new(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn touching_edges_no_overlap() {
        let a = BBox::new(0.0, 0.0, 10.0, 10.0);
        let b = BBox::new(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn within_bounds() {
        let inner = BBox::new(5.0, 5.0, 10.0, 10.0);
        let outer = BBox::new(0.0, 0.0, 100.0, 100.0);
        assert!(inner.within(&outer));
        assert!(!outer.within(&inner));
    }

    #[test]
    fn partially_outside() {
        let inner = BBox::new(-5.0, 5.0, 10.0, 10.0);
        let outer = BBox::new(0.0, 0.0, 100.0, 100.0);
        assert!(!inner.within(&outer));
    }

    #[test]
    fn line_through_box() {
        let b = BBox::new(10.0, 10.0, 20.0, 20.0);
        assert!(b.intersects_line(0.0, 0.0, 40.0, 40.0));
    }

    #[test]
    fn line_missing_box() {
        let b = BBox::new(10.0, 10.0, 20.0, 20.0);
        assert!(!b.intersects_line(0.0, 0.0, 5.0, 5.0));
    }

    #[test]
    fn line_inside_box() {
        let b = BBox::new(0.0, 0.0, 100.0, 100.0);
        assert!(b.intersects_line(10.0, 10.0, 50.0, 50.0));
    }

    #[test]
    fn horizontal_line_through() {
        let b = BBox::new(10.0, 10.0, 20.0, 20.0);
        assert!(b.intersects_line(0.0, 20.0, 50.0, 20.0));
    }

    #[test]
    fn horizontal_line_above() {
        let b = BBox::new(10.0, 10.0, 20.0, 20.0);
        assert!(!b.intersects_line(0.0, 5.0, 50.0, 5.0));
    }

    #[test]
    fn vertical_line_through() {
        let b = BBox::new(10.0, 10.0, 20.0, 20.0);
        assert!(b.intersects_line(20.0, 0.0, 20.0, 50.0));
    }
}
