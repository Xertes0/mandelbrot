extern crate num_traits;
extern crate palette;

use num_traits::Num;

use palette::{
    Hsv,
    RgbHue,
    rgb::Rgb,
    FromColor,
    Pixel,
};

pub struct Mandelbrot {
    pub pixels:   Vec::<u8>,
    pub range:    (f32,f32),
    pub pos:      (f32,f32),
    pub max_iter: u32,
    width:  u32,
    height: u32,
}

impl Mandelbrot {
    pub fn new(width: u32, height: u32) -> Self {
        Mandelbrot {
            width, height,
            pixels:   vec![0; (width*height*3) as usize], // 3 colors - RGB
            range:    (0.,0.),
            pos:      (0.,0.),
            max_iter: 0,
        }
    }

    pub fn update(&mut self) {
        let mut hsv = Hsv::new(0., 1., 0.);
        for x in 0..self.width {
            for y in 0..self.height {
                let mut a: f32 = map::<f32>(x as f32, 0., self.width  as f32, self.range.0, self.range.1) + self.pos.0;
                let mut b: f32 = map::<f32>(y as f32, 0., self.height as f32, self.range.0, self.range.1) + self.pos.1;

                let ca = a;
                let cb = b;

                let mut iter = self.max_iter;
                for i in 0..self.max_iter {
                    //println!("Iter: {}", i);
                    let aa = (a*a) - (b*b);
                    let bb = 2.*a*b;
                    a = aa+ca;
                    b = bb+cb;
                    if a+b > 16. {
                        iter = i;
                        break;
                    }
                }

                hsv.hue   = RgbHue::from_degrees(map::<f32>(iter as f32, 0., self.max_iter as f32, 0., 359.));
                hsv.value = if iter == self.max_iter {0.} else {1.};

                let rgb: [u8; 3] = Rgb::from_color(hsv).into_format().into_raw();
                let offset: usize = ((y*self.width*3) + (x*3)) as usize;
                self.pixels[offset]   = rgb[0];
                self.pixels[offset+1] = rgb[1];
                self.pixels[offset+2] = rgb[2];
            }
        }
    }
}

fn map<T: Num + Copy>(val: T, a_min: T, a_max: T, b_min: T, b_max: T) -> T {
    // Y = (X-A)/(B-A) * (D-C) + C
    (val-a_min)/(a_max-a_min) * (b_max-b_min) + b_min
}
