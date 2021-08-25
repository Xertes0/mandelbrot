extern crate sdl2;
extern crate num_traits;
extern crate palette;

use sdl2::{
    render::{
        Canvas,
        TextureCreator,
        Texture,
    },
    pixels::{
        Color,
        PixelFormatEnum,
    },
    event::Event,
    keyboard::Keycode,
    rect::Point,
    video::Window,
};

use num_traits::Num;

use palette::{
    Hsv,
    RgbHue,
    rgb::Rgb,
    FromColor,
    Pixel,
};

fn map<T: Num + Copy>(val: T, a_min: T, a_max: T, b_min: T, b_max: T) -> T {
    // Y = (X-A)/(B-A) * (D-C) + C
    (val-a_min)/(a_max-a_min) * (b_max-b_min) + b_min
}

const WIDTH:  u32 = 500;
const HEIGHT: u32 = 500;
const ZOOM_FACTOR:      f32 = 0.4;
const MOV_SPEED_FACTOR: f32 = 0.925;

fn draw_set(canvas: &mut Canvas<Window>, texture: &mut Texture, range: &(f32,f32), pos: &(f32,f32), max_iter: u32) -> Result<(), String> {
    let mut pixels: Vec<u8> = vec![0; (WIDTH*HEIGHT*3) as usize]; // 3 colors - RGB
    let mut hsv = Hsv::new(0., 1., 0.);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let mut a: f32 = map::<f32>(x as f32, 0., WIDTH  as f32, range.0, range.1) + pos.0;
            let mut b: f32 = map::<f32>(y as f32, 0., HEIGHT as f32, range.0, range.1) + pos.1;

            let ca = a;
            let cb = b;

            let mut iter = max_iter;
            for i in 0..max_iter {
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

            hsv.hue   = RgbHue::from_degrees(map::<f32>(iter as f32, 0., max_iter as f32, 0., 359.));
            hsv.value = if iter == max_iter {0.} else {1.};

            let rgb: [u8; 3] = Rgb::from_color(hsv).into_format().into_raw();
            let offset: usize = ((y*WIDTH*3) + (x*3)) as usize;
            pixels[offset]   = rgb[0];
            pixels[offset+1] = rgb[1];
            pixels[offset+2] = rgb[2];
            //canvas.set_draw_color(Color::RGB(rgb[0], rgb[1], rgb[2]));
            //canvas.draw_point(Point::new(x as i32,y as i32))?;
        }
    }

    texture.update(None, pixels.as_slice(), (WIDTH*3) as usize).unwrap(); // last parm - bytes in a row
    canvas.copy(&texture, None, None).unwrap();

    Ok(())
}

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

    let mut max_iter: u32 = 100;

    //-0.715181 -i0.230028
    //let mut range: (f32,f32) = (-0.715181,-0.230028);
    let mut range: (f32,f32) = (-2.5,1.5);
    let mut pos:   (f32,f32) = (0.,0.);

    let mut zoom:      f32 = 1.;
    let mut mov_speed: f32 = 0.4;

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
                        Some(Keycode::E) => { range.0 += ZOOM_FACTOR/zoom; range.1 -= ZOOM_FACTOR/zoom; zoom += 1.; mov_speed *= MOV_SPEED_FACTOR; },
                        Some(Keycode::Q) => { range.0 -= ZOOM_FACTOR/zoom; range.1 += ZOOM_FACTOR/zoom; zoom -= 1.; mov_speed /= MOV_SPEED_FACTOR; },
                        Some(Keycode::W) => { pos.1 -= mov_speed; },
                        Some(Keycode::S) => { pos.1 += mov_speed; },
                        Some(Keycode::A) => { pos.0 -= mov_speed; },
                        Some(Keycode::D) => { pos.0 += mov_speed; },
                        Some(Keycode::K) => { max_iter += 10; },
                        Some(Keycode::J) => { if max_iter != 0 { max_iter -= 10; } },
                        _ => {}
                    }

                    draw = true
                },
                _ => {}
            }
        }

        if draw {
            println!("Range:       {} {}", range.0, range.1);
            println!("Pos:         {} {}", pos.0, pos.1);
            println!("Zoom/factor: {} {}", zoom, ZOOM_FACTOR);
            println!("Mov/factor : {} {}", mov_speed, MOV_SPEED_FACTOR);
            println!("Max iter   : {}",    max_iter);
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            println!("drawing");

            draw_set(&mut canvas, &mut texture, &range, &pos, max_iter)?;

            println!("frame");
            canvas.present();

            draw = false;
        }

        //thread::sleep(time::Duration::from_millis(100));
    }


    Ok(())
}
