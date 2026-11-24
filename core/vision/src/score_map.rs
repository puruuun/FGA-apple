#[derive(Debug, Clone, PartialEq)]
pub struct ScoreMap {
    width: usize,
    height: usize,
    values: Vec<f32>,
}

impl ScoreMap {
    pub(crate) fn new(width: usize, height: usize, values: Vec<f32>) -> Self {
        Self {
            width,
            height,
            values,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        if x >= self.width {
            return None;
        }
        self.values.get(y * self.width + x).copied()
    }

    pub fn max_loc(&self) -> Option<(f64, usize, usize)> {
        let mut best: Option<(f32, usize)> = None;
        for (index, &value) in self.values.iter().enumerate() {
            if !value.is_nan() && best.is_none_or(|(max, _)| value > max) {
                best = Some((value, index));
            }
        }
        best.map(|(max, index)| (f64::from(max), index % self.width, index / self.width))
    }

    pub(crate) fn sanitize_masked_result(&mut self) {
        for value in &mut self.values {
            if !(-1.01_f32 <= *value && *value <= 1.01_f32) {
                *value = 0.0;
            }
        }
    }

    pub(crate) fn flood_fill(&mut self, x: usize, y: usize, max_diff: f32, new_value: f32) {
        let Some(seed) = self.get(x, y) else {
            return;
        };
        let low = -max_diff;
        let in_range = |value: f32| {
            let diff = value - seed;
            low <= diff && diff <= max_diff
        };
        let mut filled = vec![false; self.values.len()];
        let mut stack = vec![(x, y)];
        if let Some(flag) = filled.get_mut(y * self.width + x) {
            *flag = true;
        }
        while let Some((cx, cy)) = stack.pop() {
            if let Some(value) = self.values.get_mut(cy * self.width + cx) {
                *value = new_value;
            }
            let mut neighbours = [None; 4];
            if cx > 0 {
                neighbours[0] = Some((cx - 1, cy));
            }
            if cx + 1 < self.width {
                neighbours[1] = Some((cx + 1, cy));
            }
            if cy > 0 {
                neighbours[2] = Some((cx, cy - 1));
            }
            if cy + 1 < self.height {
                neighbours[3] = Some((cx, cy + 1));
            }
            for (nx, ny) in neighbours.into_iter().flatten() {
                let index = ny * self.width + nx;
                let fresh = filled.get(index).is_some_and(|flag| !flag);
                let matches = self.values.get(index).is_some_and(|&value| in_range(value));
                if fresh && matches {
                    if let Some(flag) = filled.get_mut(index) {
                        *flag = true;
                    }
                    stack.push((nx, ny));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_loc_takes_the_first_maximum_in_row_order() {
        let map = ScoreMap::new(3, 2, vec![0.1, 0.9, 0.2, 0.9, 0.3, 0.9]);
        assert_eq!(map.max_loc(), Some((f64::from(0.9_f32), 1, 0)));
    }

    #[test]
    fn sanitize_zeroes_nan_infinity_and_values_beyond_one_point_zero_one() {
        let mut map = ScoreMap::new(
            7,
            1,
            vec![f32::NAN, f32::INFINITY, -1.02, 1.0101, 1.01, -1.01, 0.5],
        );
        map.sanitize_masked_result();
        assert_eq!(map.values(), &[0.0, 0.0, 0.0, 0.0, 1.01, -1.01, 0.5]);
    }
}
