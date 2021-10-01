extern crate palette;
extern crate rayon;

use rayon::prelude::*;

use palette::{
    Hsv,
    RgbHue,
    rgb::Rgb,
    FromColor,
    Pixel,
};

#[derive(Default)]
pub struct Mandelbrot {
    pub range:    (f64,f64),
    pub pos:      (f64,f64),
    pub max_iter: u32,
    pixels:   Vec::<[u8;3]>,
    width:  u32,
    height: u32,
}

impl Mandelbrot {
    pub fn builder(width: u32, height: u32) -> MandelbrotBuilder {
        MandelbrotBuilder {
            mandelbrot: Mandelbrot {
                width, height,
                pixels:   vec![[0,0,0]; (width*height) as usize], // 3 colors - RGB
                ..Default::default()
            }
        }
    }

    pub fn get_pixels(&self) -> Vec<u8> {
        let flat = self.pixels.clone();
        flat.into_iter().flatten().collect::<Vec<u8>>()
    }

    pub fn update(&mut self) {
        let width    = self.width;
        let height   = self.height;
        let range    = self.range;
        let pos      = self.pos;
        let max_iter = self.max_iter;

        self.pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x: u32 = i as u32 % width;
            let y: u32 = i as u32 / height;
            let mut a: f64 = map::<f64>(x as f64, 0., width  as f64, range.0, range.1) + pos.0;
            let mut b: f64 = map::<f64>(y as f64, 0., height as f64, range.0, range.1) + pos.1;

            let ca = a;
            let cb = b;

            let mut iter = max_iter;
            for i in 0..max_iter {
                let aa = (a*a) - (b*b);
                let bb = 2.*a*b;
                a = aa+ca;
                b = bb+cb;
                if a+b > 16. {
                    iter = i;
                    break;
                }
            }

            let value = if iter == max_iter {0.} else {1.};
            iter      = (iter as i32 - max_iter as i32).abs() as u32;
            let hue   = RgbHue::from_degrees(map::<f32>(iter as f32, 0., max_iter as f32, 0., 359.));

            *pixel = Rgb::from_color(Hsv::new(hue, 1.0, value)).into_format().into_raw();
        });
    }
}

pub struct MandelbrotBuilder {
    mandelbrot: Mandelbrot,
}

#[allow(dead_code)]
impl MandelbrotBuilder {
    pub fn range(mut self, range: (f64, f64)) -> Self {
        self.mandelbrot.range = range;
        self
    }

    pub fn pos(mut self, pos: (f64, f64)) -> Self {
        self.mandelbrot.pos = pos;
        self
    }

    pub fn max_iter(mut self, max_iter: u32) -> Self {
        self.mandelbrot.max_iter = max_iter;
        self
    }

    pub fn build(self) -> Mandelbrot {
        self.mandelbrot
    }
}

use std::ops::{Add,Sub,Div,Mul};
fn map<T>(val: T, a_min: T, a_max: T, b_min: T, b_max: T) -> T
where
    T: Copy +
        Add<Output = T> +
        Sub<Output = T> +
        Mul<Output = T> +
        Div<Output = T>,
{
    (val-a_min)/(a_max-a_min) * (b_max-b_min) + b_min
}
