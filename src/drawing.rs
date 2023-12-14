mod circle;
mod line;

pub use circle::*;
pub use line::*;
use palette::Srgba;

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) color: Srgba,
}

// pub type Shape2D = Vec<Pixel>;
#[derive(Debug, Clone)]
pub enum Shape2D {
    Pixel(Pixel),
    Complex(Vec<Pixel>),
}

impl<I: Iterator<Item = Pixel>> From<I> for Shape2D {
    fn from(value: I) -> Self {
        Shape2D::Complex(value.collect())
    }
}

impl IntoIterator for Shape2D {
    type Item = Pixel;

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Shape2D::Pixel(p) => Box::new(std::iter::once(p)),
            Shape2D::Complex(v) => Box::new(v.into_iter()),
        }
    }
}

impl FromIterator<Pixel> for Shape2D {
    fn from_iter<T: IntoIterator<Item = Pixel>>(iter: T) -> Self {
        let mut c = vec![];
        for e in iter {
            c.push(e);
        }
        if c.len() == 1 {
            Self::Pixel(c[0])
        } else {
            Self::Complex(c)
        }
    }
}
