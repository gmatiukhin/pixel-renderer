use glam::Mat4;

#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub aperture: (u8, u8),
    pub focal_length: f32,
    pub near: f32,
    pub far: f32,
    pub fit_strategy: FitStrategy,
    /// Camera's location and rotation in the world
    pub transform: Mat4,
}

#[derive(Debug, Clone, Copy)]
pub enum FitStrategy {
    Fill,
    Overscan,
}

impl Camera {
    pub fn canvas(&self, output_dimensions: (u32, u32)) -> Canvas {
        Canvas::from_camera_parameters(
            self.aperture,
            self.focal_length,
            self.near,
            self.fit_strategy,
            output_dimensions,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Canvas {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Canvas {
    pub fn from_camera_parameters(
        aperture: (u8, u8),
        focal_length: f32,
        near: f32,
        fit_strategy: FitStrategy,
        output_dimensions: (u32, u32),
    ) -> Self {
        let aperture_aspect_ratio = aperture.0 as f32 / aperture.1 as f32;
        let output_aspect_ratio = output_dimensions.0 as f32 / output_dimensions.1 as f32;
        let (x_scale, y_scale): (f32, f32) = match fit_strategy {
            FitStrategy::Fill => {
                if aperture_aspect_ratio > output_aspect_ratio {
                    (output_aspect_ratio / aperture_aspect_ratio, 1f32)
                } else {
                    (1f32, aperture_aspect_ratio / output_aspect_ratio)
                }
            }
            FitStrategy::Overscan => {
                if aperture_aspect_ratio > output_aspect_ratio {
                    (1f32, aperture_aspect_ratio / output_aspect_ratio)
                } else {
                    (output_aspect_ratio / aperture_aspect_ratio, 1f32)
                }
            }
        };

        let width = 2f32 * (aperture.0 as f32 / 2f32 / focal_length) * near * x_scale;
        let height = width / aperture_aspect_ratio * y_scale;

        Self { width, height }
    }
}
