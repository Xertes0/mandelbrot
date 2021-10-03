extern crate rayon;

#[cfg(not(no_gpu))]
pub mod gpu;
#[cfg(not(no_gpu))]
use gpu::GpuCompute;

pub mod compute;

use rayon::prelude::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct MandelbrotParameters {
    pub range:    (f64,f64),
    pub pos:      (f64,f64),
    pub max_iter: u32,
    width:  u32,
    height: u32,
}

#[cfg(not(no_gpu))]
impl MandelbrotParameters {
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width  = width;
        self.height = height;
    }
}

pub struct Mandelbrot {
    params: MandelbrotParameters,
    pixels: Vec::<u8>,
    pub on_gpu: bool,
    #[cfg(not(no_gpu))]
    gpu_compute: Option<GpuCompute>,
}

impl Mandelbrot {
    pub fn builder(width: u32, height: u32) -> MandelbrotBuilder {
        MandelbrotBuilder {
            mandelbrot: Mandelbrot {
                params: MandelbrotParameters {
                    width, height,
                    ..Default::default()
                },
                pixels: vec![0u8;(width*height*3) as usize], // 3 colors RGB
                on_gpu: true,
                #[cfg(not(no_gpu))]
                gpu_compute: GpuCompute::new((width*height*3) as usize),
            }
        }
    }

    pub fn params(&self) -> &MandelbrotParameters {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut MandelbrotParameters {
        &mut self.params
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.pixels.resize((width*height*3) as usize, 0);
        self.params.width  = width;
        self.params.height = height;
    }

    pub fn update(&mut self) {
        #[cfg(not(no_gpu))]
        if self.on_gpu && self.gpu_compute.is_some() {
            self.update_gpu();
            return;
        }
        self.update_cpu();
    }

    #[cfg(not(no_gpu))]
    fn update_gpu(&mut self) {
        let gpu_out = self.gpu_compute.as_mut().unwrap().compute(&self.params).unwrap();
        self.pixels = gpu_out.iter().map(|x| *x as u8).collect::<Vec<u8>>();
    }
    
    fn update_cpu(&mut self) {
        let params = self.params;

        self.pixels.par_iter_mut().enumerate().step_by(3).for_each(|(i, pixel)| {
            let iter = Self::compute(&params, (i/3) as u32);
            let color = Self::color(iter, params.max_iter);

            unsafe {
                let pixel = pixel as *mut u8;

                *pixel          = color[0];
                *(pixel.add(1)) = color[1];
                *(pixel.add(2)) = color[2];
            }
        });
    }

    fn compute(params: &MandelbrotParameters, i: u32) -> u32 {
        let x: u32 = i % params.width;
        let y: u32 = i / params.height;
        let x: f64 = map::<f64>(x as f64, 0., params.width  as f64, params.range.0, params.range.1) + params.pos.0;
        let y: f64 = map::<f64>(y as f64, 0., params.height as f64, params.range.0, params.range.1) + params.pos.1;

        let mut x2: f64 = 0.;
        let mut y2: f64 = 0.;

        let mut iter = 0;
        while x2*x2+y2*y2 <= 4. && iter < params.max_iter {
            let x_new = x2*x2-y2*y2 + x;
            y2 = 2.*x2*y2 +y;
            x2 = x_new;
            iter += 1;
        }

        iter
    }

    fn color(iter: u32, max_iter: u32) -> [u8;3] {
        //let value = if iter == max_iter {0.} else {1.};
        //let iter  = (iter as i32 - max_iter as i32).abs() as u32;
        //let hue   = RgbHue::from_degrees(map::<f32>(iter as f32, 0., max_iter as f32, 0., 359.));
        let normalized = map(iter as f32, 0., max_iter as f32, 0., 1.);
        [
            (9.*(1.-normalized)*normalized*normalized*normalized*255.) as u8,
            (15.*(1.-normalized)*(1.-normalized)*normalized*normalized*255.) as u8,
            (8.5*(1.-normalized)*(1.-normalized)*(1.-normalized) * normalized*255.) as u8
        ]
    }
}

pub struct MandelbrotBuilder {
    mandelbrot: Mandelbrot,
}

impl MandelbrotBuilder {
    pub fn range(mut self, range: (f64, f64)) -> Self {
        self.mandelbrot.params.range = range;
        self
    }

    pub fn max_iter(mut self, max_iter: u32) -> Self {
        self.mandelbrot.params.max_iter = max_iter;
        self
    }

    pub fn build(self) -> Mandelbrot {
        self.mandelbrot
    }
}

impl Clone for Mandelbrot {
    fn clone(&self) -> Self {
        Self{
            params: self.params.clone(),
            pixels: self.pixels.clone(),
            on_gpu: false,
            #[cfg(not(no_gpu))]
            gpu_compute: None,
        }
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

/*
#[cfg(test)]
mod bench {
    extern crate test;
    use test::bench::Bencher;

    use super::Mandelbrot;

    const LEN: usize = (crate::WIDTH*crate::HEIGHT*3) as usize;
    #[bench]
    fn update_test(b: &mut Bencher) {
        let mut mandelbrot = Mandelbrot::builder(crate::WIDTH, crate::HEIGHT)
            .max_iter(100)
            .range((-2.5,1.5))
            .build();
        b.iter(|| {
            mandelbrot.update();
            mandelbrot.pixels();
        });
    }
}
*/
