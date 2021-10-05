//#![feature(test)]

extern crate sdl2;
extern crate image;

mod mandelbrot;
use mandelbrot::Mandelbrot;

#[cfg(feature = "gpu")]
use mandelbrot::gpu::GpuCompute;

use mandelbrot::compute::Compute;
use mandelbrot::compute::ComputeCPU;
#[cfg(feature = "gpu")]
use mandelbrot::compute::ComputeGPU;

use sdl2::{
    pixels::{
        Color,
        PixelFormatEnum,
    },
    event::Event,
    keyboard::Keycode,
};

#[cfg(feature = "gpu")]
use std::sync::{Arc,Mutex};
use std::path::Path;
use std::time::{Instant,Duration};
use std::sync::mpsc;
use std::collections::HashMap;

const WIDTH:  u32 = 1000; // Must be the same
const HEIGHT: u32 = 1000; // Must be the same
// Because wgsl dosen't support u8 we need to use u32 thus allocating 4 times more memory
// And limiting the mandelbrot to 3344 by 3344
const ALIA: u32 = 2344; // WIDTH+ALIA and HEIGHT+ALIA cannot be greater than 3344
const ZOOM_FACTOR:      f64 = 0.95;
const MOV_SPEED_FACTOR: f64 = 0.95;
const MOVEMENT_SPEED_DEFAULT: f64 = 0.5;

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
        .range((-2.5,2.5))
        .build();

    let mut zoom:      f64 = 1.;
    let mut mov_speed: f64 = MOVEMENT_SPEED_DEFAULT;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_static(PixelFormatEnum::RGB24, WIDTH, HEIGHT).unwrap();
    
    let mut alia_timer = Instant::now();

    let mut alia_rx: Option<mpsc::Receiver<Vec<u8>>> = None;
    let mut is_alia = false;

    let alia_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let mut should_alia = true;

    let mut keys_pressed = HashMap::new();

    let mut alia_on = true;

    #[cfg(feature = "gpu")]
    let alia_gpu_compute: Option<Arc<Mutex<GpuCompute>>> =
        match GpuCompute::new(((WIDTH+ALIA)*(HEIGHT+ALIA)*3) as usize) {
            Some(gpu_compute) => Some(Arc::new(Mutex::new(gpu_compute))),
            None => {
                println!("!----- GPU computing is not supported -----!");
                None
            },
        };

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyUp { keycode: key, .. } => {
                    match key {
                        Some(Keycode::E) => { keys_pressed.insert(Keycode::E, false); },
                        Some(Keycode::Q) => { keys_pressed.insert(Keycode::Q, false); },
                        Some(Keycode::W) => { keys_pressed.insert(Keycode::W, false); }, Some(Keycode::S) => { keys_pressed.insert(Keycode::S, false); },
                        Some(Keycode::A) => { keys_pressed.insert(Keycode::A, false); },
                        Some(Keycode::D) => { keys_pressed.insert(Keycode::D, false); },
                        Some(Keycode::K) => { keys_pressed.insert(Keycode::K, false); },
                        Some(Keycode::J) => { keys_pressed.insert(Keycode::J, false); },
                        _ => {}
                    }
                },
                Event::KeyDown { keycode: key, .. } => {
                    match key {
                        Some(Keycode::E) => { keys_pressed.insert(Keycode::E, true); },
                        Some(Keycode::Q) => { keys_pressed.insert(Keycode::Q, true); },
                        Some(Keycode::W) => { keys_pressed.insert(Keycode::W, true); },
                        Some(Keycode::S) => { keys_pressed.insert(Keycode::S, true); },
                        Some(Keycode::A) => { keys_pressed.insert(Keycode::A, true); },
                        Some(Keycode::D) => { keys_pressed.insert(Keycode::D, true); },
                        Some(Keycode::K) => { keys_pressed.insert(Keycode::K, true); },
                        Some(Keycode::J) => { keys_pressed.insert(Keycode::J, true); },
                        Some(Keycode::R) => { alia_on = !alia_on; should_alia = true; },
                        Some(Keycode::G) => { mandelbrot.on_gpu = !mandelbrot.on_gpu; },
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
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        for (key, pressed) in keys_pressed.iter() {
            if !pressed {continue;}
            match key {
                Keycode::E => {
                    mandelbrot.params_mut().range.0 *= ZOOM_FACTOR;
                    mandelbrot.params_mut().range.1 *= ZOOM_FACTOR;
                    zoom += 1.;
                    mov_speed *= MOV_SPEED_FACTOR;
                },
                Keycode::Q => {
                    mandelbrot.params_mut().range.0 /= ZOOM_FACTOR;
                    mandelbrot.params_mut().range.1 /= ZOOM_FACTOR;
                    zoom -= 1.;
                    mov_speed /= MOV_SPEED_FACTOR;
                },
                Keycode::W => { mandelbrot.params_mut().pos.1 -= mov_speed; },
                Keycode::S => { mandelbrot.params_mut().pos.1 += mov_speed; },
                Keycode::A => { mandelbrot.params_mut().pos.0 -= mov_speed; },
                Keycode::D => { mandelbrot.params_mut().pos.0 += mov_speed; },
                Keycode::K => { mandelbrot.params_mut().max_iter += 10; },
                Keycode::J => { if mandelbrot.params().max_iter != 0 { mandelbrot.params_mut().max_iter -= 10; } },
                _ => {}
            }
            draw = true;
        }

        if draw {
            println!("{:#?}",              mandelbrot.params());
            println!("Zoom/factor: {} {}", zoom, ZOOM_FACTOR);
            println!("Next zoom:   {}"   , ZOOM_FACTOR/zoom);
            println!("Mov/factor : {} {}", mov_speed, MOV_SPEED_FACTOR);
            canvas.set_draw_color(Color::RGB(0, 0, 0));

            println!("drawing");
            let instant = Instant::now();
            canvas.clear();

            mandelbrot.update();
            texture.update(None, mandelbrot.pixels(), (WIDTH*3) as usize).unwrap(); // last parm - bytes in a row

            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
            let elapsed = instant.elapsed();
            println!("frame");
            println!("Elapsed: {:?}", elapsed);

            draw = false;
            is_alia     = false;
            should_alia = true;
            alia_rx     = None;
            alia_timer  = Instant::now();
        } else if alia_on && !is_alia && should_alia && alia_timer.elapsed() > Duration::from_millis(500) {
            println!("!----- Thread created -----!");
            let (tx,rx) = mpsc::channel();
            alia_rx = Some(rx);

            #[cfg(feature = "gpu")]
            let mut compute: Box<dyn Compute+Send> =
                if mandelbrot.on_gpu && alia_gpu_compute.is_some() {
                    let mut params = *mandelbrot.params();
                    params.set_dimensions(WIDTH+ALIA, HEIGHT+ALIA);
                    Box::new(ComputeGPU::new(params, Arc::clone(alia_gpu_compute.as_ref().unwrap())))
                } else {
                    let mut mandelbrot_copy = mandelbrot.clone();
                    mandelbrot_copy.set_dimensions(WIDTH+ALIA, HEIGHT+ALIA);
                    Box::new(ComputeCPU::new(mandelbrot_copy))
                };
            #[cfg(not(feature = "gpu"))]
            let mut mandelbrot_copy = mandelbrot.clone();
            #[cfg(not(feature = "gpu"))]
            mandelbrot_copy.set_dimensions(WIDTH+ALIA, HEIGHT+ALIA);
            #[cfg(not(feature = "gpu"))]
            let mut compute = ComputeCPU::new(mandelbrot_copy);

            alia_pool.spawn(move|| {
                let now = Instant::now();
                let mut img = image::DynamicImage::new_rgb8(WIDTH+ALIA,HEIGHT+ALIA);

                let pixels = compute.compute();
                let mut iter = pixels.iter();
                for pixel in img.as_mut_rgb8().unwrap().pixels_mut() {
                    *pixel = image::Rgb([
                        *iter.next().unwrap(),
                        *iter.next().unwrap(),
                        *iter.next().unwrap()
                    ]);
                }
                let new = img.resize(WIDTH,HEIGHT,image::imageops::FilterType::Lanczos3);
                //let new = new.blur(0.5);
                //let unsharpen = new.unsharpen(0.5, -200);
                tx.send(new.into_bytes()).unwrap_or(());
                println!("Alia elapsed: {:?}", now.elapsed());
            });
            should_alia = false;
        } else if let Some(rx) = &alia_rx {
            if let Ok(vec) = rx.try_recv() {
                println!("!----- Received alia -----!");
                mandelbrot.set_pixels(vec);
                texture.update(None, mandelbrot.pixels(), (WIDTH*3) as usize).unwrap(); // last parm - bytes in a row
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();

                is_alia = true;
            }
        }

        std::thread::sleep(Duration::from_millis(10));
    }


    Ok(())
}
