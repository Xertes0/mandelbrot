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
                            mandelbrot.range.0 += ZOOM_FACTOR/zoom; mandelbrot.range.1 -= ZOOM_FACTOR/zoom; zoom += 1.; mov_speed *= MOV_SPEED_FACTOR;
                        },
                        Some(Keycode::Q) => {
                            mandelbrot.range.0 -= ZOOM_FACTOR/zoom; mandelbrot.range.1 += ZOOM_FACTOR/zoom; zoom -= 1.; mov_speed /= MOV_SPEED_FACTOR;
                        },
                        Some(Keycode::W) => { mandelbrot.pos.1 -= mov_speed; },
                        Some(Keycode::S) => { mandelbrot.pos.1 += mov_speed; },
                        Some(Keycode::A) => { mandelbrot.pos.0 -= mov_speed; },
                        Some(Keycode::D) => { mandelbrot.pos.0 += mov_speed; },
                        Some(Keycode::K) => { mandelbrot.max_iter += 10; },
                        Some(Keycode::J) => { if mandelbrot.max_iter != 0 { mandelbrot.max_iter -= 10; } },
                        Some(Keycode::Space) => {
                            // Screenshot
                            println!("!----- Screenshot -----!");
                            let pixels = mandelbrot.get_pixels();
                            image::save_buffer(
                                Path::new(SCREENSHOT_PATH),
                                &pixels,
                                WIDTH,
                                HEIGHT,
                                image::ColorType::Rgb8
                            ).unwrap();
                        }
                        _ => {}
                    }

                    draw = true
                },
                _ => {}
            }
        }

        if draw {
            println!("Range:       {} {}", mandelbrot.range.0, mandelbrot.range.1);
            println!("Pos:         {} {}", mandelbrot.pos.0, mandelbrot.pos.1);
            println!("Zoom/factor: {} {}", zoom, ZOOM_FACTOR);
            println!("Next zoom:   {}"   , ZOOM_FACTOR/zoom);
            println!("Mov/factor : {} {}", mov_speed, MOV_SPEED_FACTOR);
            println!("Max iter   : {}"   , mandelbrot.max_iter);
            canvas.set_draw_color(Color::RGB(0, 0, 0));

            println!("drawing");
            let instant = std::time::Instant::now();
            canvas.clear();

            mandelbrot.update();

            texture.update(None, &mandelbrot.get_pixels(), (WIDTH*3) as usize).unwrap(); // last parm - bytes in a row
            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
            let elapsed = instant.elapsed();
            println!("frame");
            println!("Elapsed: {}", elapsed.as_millis());

            draw = false;
        }

        //thread::sleep(time::Duration::from_millis(100));
    }


    Ok(())
}
