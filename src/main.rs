use std::fs::File;
use std::io::prelude::*;
use std::num::Wrapping;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

const LENGTH: u32 = 8000;
const HEIGHT: u32 = 4571;
const MAX_ITER: u32 = 1000;

fn mandelbrot(x: u32, y: u32, image: Arc<Mutex<Vec<u8>>>) {
    let x0 = (x as f32 * (1.0 - -2_f32)) / LENGTH as f32 + -2_f32;
    let y0 = (y as f32 * (1_f32 - -1_f32)) / HEIGHT as f32 + -1_f32;
    let mut mut_x = 0_f32;
    let mut mut_y = 0_f32;
    let mut iter = 0;
    while mut_x * mut_x + mut_y * mut_y <= 4_f32 && iter < MAX_ITER {
        let xtemp = mut_x * mut_x - mut_y * mut_y + x0;
        mut_y = 2_f32 * mut_x * mut_y + y0;
        mut_x = xtemp;
        iter += 1;
    }
    let green = (((iter as f32 / MAX_ITER as f32) * 255_f32) * 3_f32) as u8;
    let blue = Wrapping(Wrapping(green) * Wrapping(4)).0 .0;
    let red = ((blue as f32 / 3_f32).sin() * 255_f32) as u8;
    let mut new = image.lock().unwrap();
    new[(((y * LENGTH + x) * 3) + 0) as usize] = red;
    new[(((y * LENGTH + x) * 3) + 1) as usize] = green;
    new[(((y * LENGTH + x) * 3) + 2) as usize] = blue as u8;
}

fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let num_threads = 20;
    assert_eq!(LENGTH % num_threads, 0);
    let cols_p_t = LENGTH / num_threads; // columns per thread
    let image: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![0; (LENGTH * HEIGHT * 3) as usize]));

    let bmp: [u8; 14] = [
        0x42, 0x4D, // BMP ID
        0x76, 0xF3, 0x89, 0x06, // Total size of file
        0x00, 0x00, 0x00, 0x00, // unused
        0x36, 0x00, 0x00, 0x00, // offset where image starts
    ];

    // set the dib header array
    let dib: [u8; 36] = [
        0x28, 0x00, 0x00, 0x00, // number of bytes in dib header
        0x40, 0x1F, 0x00, 0x00, // Length in pixels
        0xDB, 0x11, 0x00, 0x00, // Height in pixels
        0x01, 0x00, // number of color planes
        0x18, 0x00, // number of bits per pixel
        0x00, 0x00, 0x00, 0x00, // compression
        0x40, 0xF3, 0x89, 0x06, // size of image data in bytes
        0x13, 0x0B, 0x00, 0x00, // print resoolution
        0x00, 0x00, 0x00, 0x00, // print resolution
        0x00, 0x00, 0x00, 0x00,
    ];

    let handles: Vec<_> = (0..num_threads)
        .map(|thread| {
            let image_clone = Arc::clone(&image);
            thread::spawn(move || {
                for x in (thread * cols_p_t)..((cols_p_t * thread) + cols_p_t) {
                    for y in 0..HEIGHT {
                        mandelbrot(x, y, Arc::clone(&image_clone));
                    }
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();
    println!("{:?}", duration);
    let mut file = File::create("images/test.bmp")?;
    file.write_all(&bmp)?;
    file.write_all(&dib)?;
    file.write_all(&*image.lock().unwrap())?;
    Ok(())
}
