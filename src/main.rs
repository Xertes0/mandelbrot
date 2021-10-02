//#![feature(test)]

extern crate sdl2;
extern crate image;

mod mandelbrot;
use mandelbrot::Mandelbrot;

use sdl2::{
    pixels::{
        Color,
        PixelFormatEnum,
    },
    event::Event,
    keyboard::Keycode,
};

use std::path::Path;

const WIDTH:  u32 = 1000; // Must be the same
const HEIGHT: u32 = 1000; // Must be the same
const ALIA:   u32 = 3000;
const ZOOM_FACTOR:      f64 = 0.4;
const MOV_SPEED_FACTOR: f64 = 0.930;

const SCREENSHOT_PATH: &str = "./screenshot.png";

fn main() -> Result<(), String> {
    println!("Hello, world!");
    
    let sdl_context = sdl2::init()?;
    let vid_subsys = sdl_context.video()?;

    let window = vid_subsys
        .window("Mandelbrot Set", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut draw = true;

    let mut mandelbrot = Mandelbrot::builder(WIDTH, HEIGHT)
        .max_iter(100)
        .range((-2.5,1.5))
        .build();

    let mut zoom:      f64 = 1.;
    let mut mov_speed: f64 = 0.4;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_static(PixelFormatEnum::RGB24, WIDTH, HEIGHT).unwrap();

    let mut img = image::DynamicImage::new_rgb8(WIDTH+ALIA,HEIGHT+ALIA);

    let mut alia_on = false;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: key,
                    ..
                } => {
                    match key {
                        Some(Keycode::E) => {
                            mandelbrot.params_mut().range.0 += ZOOM_FACTOR/zoom; mandelbrot.params_mut().range.1 -= ZOOM_FACTOR/zoom; zoom += 1.; mov_speed *= MOV_SPEED_FACTOR;
                        },
                        Some(Keycode::Q) => {
                            mandelbrot.params_mut().range.0 -= ZOOM_FACTOR/zoom; mandelbrot.params_mut().range.1 += ZOOM_FACTOR/zoom; zoom -= 1.; mov_speed /= MOV_SPEED_FACTOR;
                        },
                        Some(Keycode::W) => { mandelbrot.params_mut().pos.1 -= mov_speed; },
                        Some(Keycode::S) => { mandelbrot.params_mut().pos.1 += mov_speed; },
                        Some(Keycode::A) => { mandelbrot.params_mut().pos.0 -= mov_speed; },
                        Some(Keycode::D) => { mandelbrot.params_mut().pos.0 += mov_speed; },
                        Some(Keycode::K) => { mandelbrot.params_mut().max_iter += 10; },
                        Some(Keycode::J) => { if mandelbrot.params().max_iter != 0 { mandelbrot.params_mut().max_iter -= 10; } },
                        Some(Keycode::Space) => {
                            println!("!----- Screenshot -----!");
                            let pixels = mandelbrot.pixels();
                            image::save_buffer(
                                Path::new(SCREENSHOT_PATH),
                                pixels,
                                WIDTH,
                                HEIGHT,
                                image::ColorType::Rgb8
                            ).unwrap();
                        },
                        Some(Keycode::F) => { alia_on = !alia_on; },
                        _ => {}
                    }

                    draw = true
                },
                _ => {}
            }
        }

        if draw {
            println!("{:#?}",              mandelbrot.params());
            println!("Zoom/factor: {} {}", zoom, ZOOM_FACTOR);
            println!("Next zoom:   {}"   , ZOOM_FACTOR/zoom);
            println!("Mov/factor : {} {}", mov_speed, MOV_SPEED_FACTOR);
            canvas.set_draw_color(Color::RGB(0, 0, 0));

            println!("drawing");
            let instant = std::time::Instant::now();
            canvas.clear();

            if alia_on {
                mandelbrot.set_dimension(WIDTH+ALIA,HEIGHT+ALIA);
                mandelbrot.update();

                let mut iter = mandelbrot.pixels().iter();
                for pixel in img.as_mut_rgb8().unwrap().pixels_mut() {
                    *pixel = image::Rgb([*iter.next().unwrap(),*iter.next().unwrap(),*iter.next().unwrap()]);
                }
                let new = img.resize(WIDTH,HEIGHT,image::imageops::FilterType::Lanczos3);
                //let new = new.blur(0.5);
                //let unsharpen = new.unsharpen(0.5, -200);
                let flat = new.as_flat_samples_u8().unwrap();
                texture.update(None, flat.as_slice(), (WIDTH*3) as usize).unwrap(); // last parm - bytes in a row
            } else {
                mandelbrot.set_dimension(WIDTH,HEIGHT);
                mandelbrot.update();
                texture.update(None, mandelbrot.pixels(), (WIDTH*3) as usize).unwrap(); // last parm - bytes in a row
            }

            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
            let elapsed = instant.elapsed();
            println!("frame");
            println!("Elapsed: {}", elapsed.as_millis());

            draw = false;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }


    Ok(())
}
