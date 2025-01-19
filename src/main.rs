extern crate image;

use image::{Rgb, RgbImage};
use rand_distr::{Normal, Distribution};


fn read_width_height() -> (u32, u32) {
    let args: Vec<String> = std::env::args().collect();
    let mut height = None;
    let mut width = None;
    let default_resolution = (1920, 1080);

    if args.len() == 2 {
        width = args[0].parse().ok();
        height = args[1].parse().ok();
    };

    if let Some(width) = width {
        if let Some(height) = height {
            (width, height)
        } else {
            default_resolution
        }
    } else {
         default_resolution
    }
}

fn main() {
    let (width, height) = read_width_height();

    let mut image1 = compute_noisy_gradient(
        width, height, 
        Rgb::from([5, 47, 95]),
        Rgb::from([6, 167, 125]),
        false
    );
    let image2 = compute_noisy_gradient(
        width, height, 
        Rgb::from([186, 46, 55]),//Rgb::from([249, 62, 75]),//Rgb::from([234, 82, 111]),
        Rgb::from([180, 121, 6]),//Rgb::from([241, 162, 8]),
        true
    );


    let o_x = width as isize / 2;
    let o_y = height as isize / 2;
    let r = (3 * std::cmp::min(height, width) / 8) as f64;

    for x in 0..width {
        for y in 0..height {
            if (((o_x - x as isize)*(o_x - x as isize) + (o_y - y as isize)*(o_y - y as isize)) as f64).sqrt() < r {
                image1.put_pixel(x, y, (*image2.get_pixel(x, y)).into());
            }
        }
    }


    let mut rng = rand::rng();
    let rand_x = Normal::new((width / 2) as f64, (width / 4) as f64).unwrap();
    let rand_y = Normal::new((height / 2) as f64, (height / 4) as f64).unwrap();

    let r = std::cmp::min(width, height) as f64 * 0.01;
    let rand_off = Normal::new(0.0, r).unwrap();

    let swap_density = 20;
    for _ in 0..(width * height * swap_density / 100){
        let x1 = (rand_x.sample(&mut rng) as u32).clamp(0, width - 1);
        let y1 = (rand_y.sample(&mut rng) as u32).clamp(0, height - 1);
    
        let off_x = rand_off.sample(&mut rng) as i32;
        let off_y = rand_off.sample(&mut rng) as i32;
        let x2 = ((x1 as i32 + off_x) as u32).clamp(0, width - 1);
        let y2 = ((y1 as i32 + off_y) as u32).clamp(0, height - 1);

        let tmp_pixel = (*image1.get_pixel(x1, y1)).into();
        image1.put_pixel(x1, y1, (*image1.get_pixel(x2, y2)).into());
        image1.put_pixel(x2, y2, tmp_pixel);
    }




    // write it out to a file
    image1.save("output.png").unwrap();
}

fn compute_noisy_gradient(width: u32, height: u32, color1: Rgb<u8>, color2: Rgb<u8>, use_minor_diag: bool) -> RgbImage {
    let swap_density = 25;

    let mut image = RgbImage::new(width, height);
    let max_dist = ((width * width + height * height) as f64).sqrt();

    for x in 0..width {
        for y in 0..height {
            let dist = if use_minor_diag {
                (((width - x) * (width - x) + y * y) as f64).sqrt() / max_dist
            } else {
                (((width - x) * (width - x) + (height - y) * (height - y)) as f64).sqrt() / max_dist
            };
            image.put_pixel(x, y, Rgb::from([
                (color1.0[0] as f64 * dist + color2.0[0] as f64 * (1.0 - dist)) as u8,
                (color1.0[1] as f64 * dist + color2.0[1] as f64 * (1.0 - dist)) as u8,
                (color1.0[2] as f64 * dist + color2.0[2] as f64 * (1.0 - dist)) as u8,
            ]));
        }
    }

    let rand_x = Normal::new((width / 2) as f64, (width / 4) as f64).unwrap();
    let rand_y = Normal::new((height / 2) as f64, (height / 4) as f64).unwrap();

    let mut rng = rand::rng();

    for _ in 0..(width * height * swap_density / 100){
        let x1 = (rand_x.sample(&mut rng) as u32).clamp(0, width - 1);
        let y1 = (rand_y.sample(&mut rng) as u32).clamp(0, height - 1);
    
        let x2 = (rand_x.sample(&mut rng) as u32).clamp(0, width - 1);
        let y2 = (rand_y.sample(&mut rng) as u32).clamp(0, height - 1);

        let tmp_pixel = (*image.get_pixel(x1, y1)).into();
        image.put_pixel(x1, y1, (*image.get_pixel(x2, y2)).into());
        image.put_pixel(x2, y2, tmp_pixel);
    }

    image
}
