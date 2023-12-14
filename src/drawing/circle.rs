use palette::Srgba;

use super::{Pixel, Shape2D};

pub trait Circle: Iterator<Item = Pixel> {
    fn new(c: (i32, i32), r: i32, color: Srgba) -> Self
    where
        Self: Sized;
}

pub struct BresenhamCircle {
    center: (i32, i32),
    color: Srgba,
    x: i32,
    y: i32,
    d: i32,
    buffer: Vec<Pixel>,
}

impl BresenhamCircle {
    #[rustfmt::skip]
    fn put_pixels(&self) -> [Pixel; 8] {
        let c = self.center;
        let x = self.x;
        let y = self.y;
        let color = self.color;
        [
            Pixel { x: c.0 + x, y: c.1 + y, color },
            Pixel { x: c.0 - x, y: c.1 + y, color },
            Pixel { x: c.0 + x, y: c.1 - y, color },
            Pixel { x: c.0 - x, y: c.1 - y, color },
            Pixel { x: c.0 + y, y: c.1 + x, color },
            Pixel { x: c.0 - y, y: c.1 + x, color },
            Pixel { x: c.0 + y, y: c.1 - x, color },
            Pixel { x: c.0 - y, y: c.1 - x, color },
        ]
    }
}

impl Circle for BresenhamCircle {
    fn new(c: (i32, i32), r: i32, color: Srgba) -> Self
    where
        Self: Sized,
    {
        Self {
            center: c,
            color,
            x: 0,
            y: r,
            d: 3 - 2 * r,
            buffer: vec![],
        }
    }
}

impl Iterator for BresenhamCircle {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.buffer.is_empty() {
            self.buffer.pop()
        } else if self.y < self.x {
            None
        } else {
            let pixels = self.put_pixels();
            self.buffer.extend_from_slice(&pixels[1..]);

            self.x += 1;
            self.d += if self.d > 0 {
                self.y -= 1;
                4 * (self.x - self.y) + 10
            } else {
                4 * self.x + 6
            };

            Some(pixels[0])
        }
    }
}
