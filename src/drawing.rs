use std::marker::PhantomData;

use palette::{Srgb, Srgba, WithAlpha};

pub type Shape2D = Vec<Pixel>;

pub trait Line: Iterator<Item = Pixel> {
    fn new(from: (i32, i32), to: (i32, i32), color: Srgb) -> Self
    where
        Self: Sized;
}

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) color: Srgba,
}

#[derive(Debug)]
pub struct NoPoints;
#[derive(Debug)]
pub struct HasStart;
#[derive(Debug)]
pub struct HasEnd;

pub struct LineBuilder<Line, Valid = ()> {
    path: Vec<(i32, i32)>,
    skip_line: Vec<(usize, usize)>,
    last_line_beginning: usize,
    color: Srgb,
    _line: PhantomData<Line>,
    _line_state: PhantomData<Valid>,
}

impl<L, V> std::fmt::Debug for LineBuilder<L, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineBuilder")
            .field("path", &self.path)
            .field("skip_line", &self.skip_line)
            .field("last_line_beginning", &self.last_line_beginning)
            .finish()
    }
}

impl<L: Line, S> LineBuilder<L, S> {
    pub fn new() -> LineBuilder<L, NoPoints> {
        LineBuilder {
            path: vec![],
            skip_line: vec![],
            last_line_beginning: 0,
            color: Srgb::new(1f32, 1f32, 1f32),
            _line: PhantomData,
            _line_state: PhantomData,
        }
    }

    pub fn color(mut self, color: Srgb) -> Self {
        self.color = color;
        self
    }
}

impl<L: Line> LineBuilder<L, NoPoints> {
    /// Starts a new line from `p`
    pub fn from(self, p: (i32, i32)) -> LineBuilder<L, HasStart> {
        LineBuilder {
            path: vec![p],
            skip_line: vec![],
            last_line_beginning: 0,
            color: self.color,
            _line: PhantomData,
            _line_state: PhantomData,
        }
    }
}

impl<L: Line> LineBuilder<L, HasStart> {
    pub fn to(self, p: (i32, i32)) -> LineBuilder<L, HasEnd> {
        let mut path = self.path;
        path.push(p);
        LineBuilder {
            path,
            skip_line: self.skip_line,
            last_line_beginning: self.last_line_beginning,
            color: self.color,
            _line: self._line,
            _line_state: PhantomData,
        }
    }
}

impl<L: Line> LineBuilder<L, HasEnd> {
    /// Draws line to `p`
    pub fn to(mut self, p: (i32, i32)) -> LineBuilder<L, HasEnd> {
        self.path.push(p);
        self
    }

    /// Moves to `p` without drawind and starts a new line
    pub fn from(mut self, p: (i32, i32)) -> LineBuilder<L, HasEnd> {
        self.path.push(p);
        self.skip_line
            .push((self.path.len() - 2, self.path.len() - 1));
        self.last_line_beginning = self.path.len() - 1;
        self
    }

    /// Draws a line between the last point and the first one.
    pub fn close(mut self) -> LineBuilder<L, HasEnd> {
        self.path.push(self.path[self.last_line_beginning]);
        self
    }

    /// Consumes the builder and returns an iterator over line pixels.
    pub fn end(self) -> impl Iterator<Item = Pixel> {
        self.path
            .clone()
            .into_iter()
            .enumerate()
            .zip(self.path.into_iter().enumerate().skip(1))
            .filter(move |((i0, _), (i1, _))| !self.skip_line.contains(&(*i0, *i1)))
            .flat_map(move |((_, p0), (_, p1))| L::new(p0, p1, self.color))
    }

    /// Returns a `Shape2D` formed by the line pixels
    pub fn shape(self) -> Shape2D {
        self.end().collect()
    }
}

#[derive(Debug)]
pub struct BresenhamLine {
    p: (i32, i32),
    to: (i32, i32),
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    error: i32,
    stop: bool,
    color: Srgb,
}

impl Line for BresenhamLine {
    fn new(from: (i32, i32), to: (i32, i32), color: Srgb) -> Self {
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
            color,
        }
    }
}

impl Iterator for BresenhamLine {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop {
            return None;
        }

        let output = Pixel {
            x: self.p.0,
            y: self.p.1,
            color: self.color.with_alpha(1f32),
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

#[derive(Debug)]
pub struct WuLine {
    steep: bool,
    x: i32,
    x_end: i32,
    inter_y: f32,
    gradient: f32,
    is_drawind_pixel_a: bool,
    start_end_buffer: Vec<Pixel>,
    color: Srgb,
}

impl Line for WuLine {
    fn new(from: (i32, i32), to: (i32, i32), color: Srgb) -> Self {
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
                Pixel {
                    x: y_start,
                    y: x_start,
                    color: color.with_alpha((1f32 - y.fract()) * x_gap),
                },
                Pixel {
                    x: y_start + 1,
                    y: x_start,
                    color: color.with_alpha(y.fract() * x_gap),
                },
            )
        } else {
            (
                Pixel {
                    x: x_start,
                    y: y_start,
                    color: color.with_alpha((1f32 - y.fract()) * x_gap),
                },
                Pixel {
                    x: x_start,
                    y: y_start + 1,
                    color: color.with_alpha(y.fract() * x_gap),
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
                Pixel {
                    x: y_end,
                    y: x_end,
                    color: color.with_alpha((1f32 - y.fract()) * x_gap),
                },
                Pixel {
                    x: y_end + 1,
                    y: x_end,
                    color: color.with_alpha(y.fract() * x_gap),
                },
            )
        } else {
            (
                Pixel {
                    x: x_end,
                    y: y_end,
                    color: color.with_alpha((1f32 - y.fract()) * x_gap),
                },
                Pixel {
                    x: x_end,
                    y: y_end + 1,
                    color: color.with_alpha(y.fract() * x_gap),
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
            color,
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
                Some(Pixel {
                    x: self.inter_y as i32,
                    y: self.x,
                    color: self.color.with_alpha(f_opacity),
                })
            } else {
                Some(Pixel {
                    x: self.x,
                    y: self.inter_y as i32,
                    color: self.color.with_alpha(f_opacity),
                })
            }
        } else {
            self.is_drawind_pixel_a = true;
            let x = self.x;
            let f_opacity = self.inter_y.fract();
            self.x += 1;
            self.inter_y += self.gradient;

            if self.steep {
                Some(Pixel {
                    x: self.inter_y as i32 + 1,
                    y: x,
                    color: self.color.with_alpha(f_opacity),
                })
            } else {
                Some(Pixel {
                    x,
                    y: self.inter_y as i32 + 1,
                    color: self.color.with_alpha(f_opacity),
                })
            }
        }
    }
}
