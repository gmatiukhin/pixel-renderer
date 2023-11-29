pub struct BresenhamLine {
    p: (i32, i32),
    to: (i32, i32),
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    error: i32,
    stop: bool,
}

impl BresenhamLine {
    pub fn new(from: (i32, i32), to: (i32, i32)) -> Self {
        let dx = (to.0 - from.0).abs();
        let sx = if from.0 < to.0 { 1 } else { -1 };
        let dy = -(to.1 - from.1).abs();
        let sy = if from.1 < to.1 { 1 } else { -1 };
        let error = dx + dy;
        Self {
            p: from,
            to,
            dx,
            dy,
            sx,
            sy,
            error,
            stop: false,
        }
    }
}

impl Iterator for BresenhamLine {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop {
            return None;
        }

        let output = self.p;

        self.stop = self.p == self.to;

        let e2 = self.error * 2;
        if e2 >= self.dy {
            self.stop = self.p.0 == self.to.0;
            self.error += self.dy;
            self.p.0 += self.sx;
        }
        if e2 <= self.dx {
            self.stop = self.p.1 == self.to.1;
            self.error += self.dx;
            self.p.1 += self.sy;
        }
        Some(output)
    }
}
