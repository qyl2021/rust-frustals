mod domain;
mod fractals;
mod pixel;
mod point;
use wasm_bindgen::prelude::*;

pub use crate::domain::Domain;
pub use crate::fractals::{Colorization, Fractal, Options, Variant};
pub use crate::pixel::Pixel;
pub use crate::point::Point;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct DomainOption {
    pub origin: Point,
    pub scale: f64,
}

#[wasm_bindgen]
pub struct Frustal {
    data: Vec<Pixel>,
    preview_data: Vec<Pixel>,
    pub options: Options,
    domain: Domain,
    fractal: Box<dyn Fractal>,
    preview_resolution: usize,
}

#[wasm_bindgen]
impl Frustal {
    pub fn new(width: usize, height: usize, preview_resolution: usize) -> Frustal {
        let variant = Variant::Mandelbrot;
        Frustal {
            data: (0..width * height).map(|_| Pixel::void()).collect(),
            preview_data: (0..(width * height) / (preview_resolution * preview_resolution))
                .map(|_| Pixel::void())
                .collect(),
            options: Options {
                variant,
                smooth: true,
                precision: 25,
                order: 2,
                const_real: 0.0,
                const_imaginary: 0.0,
                colorization: Colorization::Relative,
                black_threshold: 0.1,
                white_threshold: 0.9,
                overexposure: 1.,
                color_rotation: 0.1,
                color_base: 0.1,
            },
            domain: Domain::new(Point::new(width as f64, height as f64)),
            fractal: variant.get_fractal(),
            preview_resolution,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.domain.resize(Point::new(width as f64, height as f64));
        self.data = (0..width * height).map(|_| Pixel::void()).collect();
        self.preview_data = (0..(width * height)
            / (self.preview_resolution * self.preview_resolution))
            .map(|_| Pixel::void())
            .collect();
    }

    pub fn resize_preview(&mut self, preview_resolution: usize) {
        self.preview_resolution = preview_resolution;
        self.preview_data = (0..((self.domain.size.x as usize) * (self.domain.size.y as usize))
            / (self.preview_resolution * self.preview_resolution))
            .map(|_| Pixel::void())
            .collect();
    }

    pub fn reset_data(&mut self) {
        for point in &mut self.data {
            *point = Pixel::void()
        }
    }

    pub fn shift_domain(&mut self, point: Point) {
        self.domain.shift(point)
    }

    pub fn change_domain(&mut self, x: f64, y: f64, scale: f64) {
        self.domain.change(Point::new(x, y), scale)
    }

    pub fn zoom_domain(&mut self, factor: f64, center: Point) {
        self.domain.zoom(factor, center)
    }

    pub fn data_ptr(&mut self) -> *const Pixel {
        self.data.as_ptr()
    }

    pub fn preview_data_ptr(&mut self) -> *const Pixel {
        self.preview_data.as_ptr()
    }

    pub fn render(&mut self) {
        for (i, point) in self.domain.iter().enumerate() {
            let pixel = self.fractal.get_pixel_at_point(point, &self.options);
            self.data[i].from(pixel);
        }
    }

    pub fn preview_render(&mut self) {
        set_panic_hook();
        let width = self.domain.size.x as usize;
        let height = self.domain.size.y as usize;
        let len = (width * height) / (self.preview_resolution * self.preview_resolution);
        let mut j = 0;
        for (i, point) in self.domain.iter().enumerate() {
            if i % self.preview_resolution != 0
                || ((i + self.preview_resolution - 1) / width) % self.preview_resolution != 0
                || j >= len
            {
                continue;
            }
            let pixel = self.fractal.get_pixel_at_point(point, &self.options);
            self.preview_data[j].from(pixel);
            j += 1;
        }
    }

    pub fn partial_render(&mut self, skip: usize, index: usize) {
        for (i, point) in self.domain.iter().enumerate() {
            if (i + index) % skip != 0 {
                continue;
            }
            let pixel = self.fractal.get_pixel_at_point(point, &self.options);
            self.data[i].from(pixel);
        }
    }

    pub fn sync_options(&mut self, options: &Options) {
        self.options = *options;
        self.fractal = self.options.variant.get_fractal();
    }

    pub fn current_options(&self) -> Options {
        self.options
    }

    pub fn current_domain(&self) -> DomainOption {
        DomainOption {
            origin: self.domain.origin,
            scale: self.domain.scale,
        }
    }
}
