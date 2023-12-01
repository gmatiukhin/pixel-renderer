use std::marker::PhantomData;

pub type Shape2D = Vec<Pixel>;

pub trait Line: Iterator<Item = Pixel> {
    fn new(from: (i32, i32), to: (i32, i32)) -> Self
    where
        Self: Sized;
}

#[derive(Clone, Copy)]
pub enum Pixel {
    Normal { x: i32, y: i32 },
    AntiAliased { x: i32, y: i32, a: u8 },
}

pub struct HasStart;
pub struct HasEnd;

pub struct LineBuilder<Line, Valid = ()> {
    path: Vec<(i32, i32)>,
    _line: PhantomData<Line>,
    _is_line_valid: PhantomData<Valid>,
}

impl<L: Line, S> LineBuilder<L, S> {
    pub fn start(x: i32, y: i32) -> LineBuilder<L, HasStart> {
        LineBuilder {
            path: vec![(x, y)],
            _line: PhantomData,
            _is_line_valid: PhantomData,
        }
    }
}

impl<L: Line> LineBuilder<L, HasStart> {
    pub fn to(self, x: i32, y: i32) -> LineBuilder<L, HasEnd> {
        let mut path = self.path;
        path.push((x, y));
        LineBuilder {
            path,
            _line: PhantomData,
            _is_line_valid: PhantomData,
        }
    }
}

impl<L: Line> LineBuilder<L, HasEnd> {
    pub fn to(mut self, x: i32, y: i32) -> LineBuilder<L, HasEnd> {
        self.path.push((x, y));
        self
    }

    /// Consumes the builder and returns an iterator over line pixels.
    pub fn end(self) -> impl Iterator<Item = Pixel> {
        self.path
            .clone()
            .into_iter()
            .zip(self.path.into_iter().skip(1))
            .flat_map(|(p0, p1)| L::new(p0, p1))
    }

    /// Consumes the builder and returns an iterator over line pixels.
    /// Additionally creates a line between the last point and the first one.
    pub fn close(mut self) -> impl Iterator<Item = Pixel> {
        self.path.push(self.path[0]);
        self.end()
    }
}

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

impl Line for BresenhamLine {
    fn new(from: (i32, i32), to: (i32, i32)) -> Self {
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
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop {
            return None;
        }

        let output = Pixel::Normal {
            x: self.p.0,
            y: self.p.1,
        };

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

pub struct WuLine {
    steep: bool,
    x: i32,
    x_end: i32,
    inter_y: f32,
    gradient: f32,
    is_drawind_pixel_a: bool,
    start_end_buffer: Vec<Pixel>,
}

impl Line for WuLine {
    fn new(from: (i32, i32), to: (i32, i32)) -> Self {
        let from = (from.0 as f32, from.1 as f32);
        let to = (to.0 as f32, to.1 as f32);

        let steep = (to.1 - from.1).abs() > (to.0 - from.0).abs();

        let (from, to) = if steep {
            ((from.1, from.0), (to.1, to.0))
        } else {
            (from, to)
        };

        let (from, to) = if from.0 > to.0 {
            (to, from)
        } else {
            (from, to)
        };

        let dx = to.0 - from.0;
        let dy = to.1 - from.1;

        let gradient = if dx == 0f32 { 1f32 } else { dy / dx };

        let mut start_end_buffer = vec![];

        // Starting point
        let x = from.0.ceil();
        let y = from.1 + gradient * (x - from.0);
        let x_gap = 1f32 - (from.0 + 0.5).fract();
        let x_start = x as i32;
        let y_start = y as i32;

        let (p1, p2) = if steep {
            (
                Pixel::AntiAliased {
                    x: y_start,
                    y: x_start,
                    a: alpha_f32_to_u8((1.0 - y.fract()) * x_gap),
                },
                Pixel::AntiAliased {
                    x: y_start + 1,
                    y: x_start,
                    a: alpha_f32_to_u8(y.fract() * x_gap),
                },
            )
        } else {
            (
                Pixel::AntiAliased {
                    x: x_start,
                    y: y_start,
                    a: alpha_f32_to_u8((1.0 - y.fract()) * x_gap),
                },
                Pixel::AntiAliased {
                    x: x_start,
                    y: y_start + 1,
                    a: alpha_f32_to_u8(y.fract() * x_gap),
                },
            )
        };
        start_end_buffer.push(p1);
        start_end_buffer.push(p2);

        let inter_y = y + gradient;

        // Ending point
        let x = to.0.ceil();
        let y = to.1 + gradient * (x - to.0);
        let x_gap = (to.0 + 0.5).fract();
        let x_end = x as i32;
        let y_end = y as i32;

        let (p1, p2) = if steep {
            (
                Pixel::AntiAliased {
                    x: y_end,
                    y: x_end,
                    a: alpha_f32_to_u8((1.0 - y.fract()) * x_gap),
                },
                Pixel::AntiAliased {
                    x: y_end + 1,
                    y: x_end,
                    a: alpha_f32_to_u8(y.fract() * x_gap),
                },
            )
        } else {
            (
                Pixel::AntiAliased {
                    x: x_end,
                    y: y_end,
                    a: alpha_f32_to_u8((1.0 - y.fract()) * x_gap),
                },
                Pixel::AntiAliased {
                    x: x_end,
                    y: y_end + 1,
                    a: alpha_f32_to_u8(y.fract() * x_gap),
                },
            )
        };
        start_end_buffer.push(p1);
        start_end_buffer.push(p2);

        Self {
            steep,
            x: x_start + 1,
            x_end,
            inter_y,
            gradient,
            is_drawind_pixel_a: true,
            start_end_buffer,
        }
    }
}

impl Iterator for WuLine {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        // Output buffered endpoints
        if !self.start_end_buffer.is_empty() {
            return self.start_end_buffer.pop();
        }

        if self.x >= self.x_end {
            return None;
        }

        // Main loop
        if self.is_drawind_pixel_a {
            self.is_drawind_pixel_a = false;
            let f_opacity = 1f32 - self.inter_y.fract();

            if self.steep {
                Some(Pixel::AntiAliased {
                    x: self.inter_y as i32,
                    y: self.x,
                    a: alpha_f32_to_u8(f_opacity),
                })
            } else {
                Some(Pixel::AntiAliased {
                    x: self.x,
                    y: self.inter_y as i32,
                    a: alpha_f32_to_u8(f_opacity),
                })
            }
        } else {
            self.is_drawind_pixel_a = true;
            let x = self.x;
            let f_opacity = self.inter_y.fract();
            self.x += 1;
            self.inter_y += self.gradient;

            if self.steep {
                Some(Pixel::AntiAliased {
                    x: self.inter_y as i32 + 1,
                    y: x,
                    a: alpha_f32_to_u8(f_opacity),
                })
            } else {
                Some(Pixel::AntiAliased {
                    x,
                    y: self.inter_y as i32 + 1,
                    a: alpha_f32_to_u8(f_opacity),
                })
            }
        }
    }
}

fn alpha_f32_to_u8(f: f32) -> u8 {
    (if f >= 1f32 { 255f32 } else { f * 256f32 }) as u8
}
