# Mandelbrot Explorer
Mandelbrot explorer written in rust.

## Sample screenshot
![Sample screenshot](https://github.com/Xertes0/mandelbrot/blob/b1ca60d08df78fd4707001fd6a4ce6e8da397adb/sample_screenshots/sample.png)

## Keybinds
| Key | Action |
| :---: | --- |
| WSAD | Movement |
| E/Q | Zoom in/out |
| R | Toogle 'anit-aliasing' |
| G | Toogle gpu computing |

## Compiling from source
Rustc and cargo will be needed, you can install it with [rustup.](https://rustup.rs/)


Additionaly you will need sdl2 library on your computer.\
If you are using linux based distribution it should be in your distro's repositories.\
For windows users refer to [this instructions.](https://crates.io/crates/sdl2#windows-msvc)

### Clone the repository
```bash
git clone https://github.com/Xertes0/mandelbrot.git
cd mandelbrot
```

### Compile and run
```bash
cargo run # or 'cargo r' for short
```

#### To compile and run without gpu computing support
```bash
cargo r --no-default-features
```
