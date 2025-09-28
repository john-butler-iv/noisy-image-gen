pub mod shapes;
pub mod noise;
pub mod coloring;

use image::{RgbImage, ImageBuffer};
use shapes::CheckInside;
use coloring::{Coloring, TransparentColor};

pub struct Image {
    canvas_width: usize,
    canvas: Vec<coloring::SolidColor>,
}

pub struct DrawInstruction<R: rand::Rng> {
    pub pre_clip_noise: Option<Box<dyn noise::Noise<R>>>,
    pub clipping_shape: shapes::Shape,
    pub coloring: coloring::ColorScheme<coloring::TransparentColor>,
    pub post_clip_noise: Option<Box<dyn noise::Noise<R>>>,
    pub post_draw_noise: Option<Box<dyn noise::Noise<R>>>,
}


impl Image {
    pub fn with_size(width: usize, height: usize, background_color: coloring::SolidColor) -> Self {
        Image { 
            canvas_width: width,
            canvas: vec![background_color; width * height],
        }
    }

    fn canvas_height(&self) -> usize {
        self.canvas.len() / self.canvas_width
    }
    fn get_index(&self, x: usize, y: usize) -> usize {
        x + y * self.canvas_width
    }
    
    pub fn get_pixel(&self, x: usize, y: usize) -> &coloring::SolidColor {
        return &self.canvas[self.get_index(x, y)]
    }
    
    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut coloring::SolidColor {
        let index = self.get_index(x, y);
        return &mut self.canvas[index]
    }
    
    pub fn swap_pixels(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        let tmp_pixel = self.get_pixel(x1 as usize, y1 as usize).to_owned();
        *self.get_pixel_mut(x1 as usize, y1 as usize) = self.get_pixel(x2 as usize, y2 as usize).to_owned();
        *self.get_pixel_mut(x2 as usize, y2 as usize) = tmp_pixel;
    }
    
    pub fn output_to_image(&self, filename: &str)-> Result<(),()>{
        let image:RgbImage = ImageBuffer::from_raw(
            self.canvas_width.try_into().map_err(|_|())?,
            self.canvas_height().try_into().map_err(|_|())?,
            self.canvas.iter().map(|color| [color.red, color.green, color.blue]).collect::<Vec<[u8;3]>>().into_iter().flatten().collect())
        .expect("Image values have a width/height that matches the canvas size");
        
        image.save(filename).map_err(|_|())
    }
}

impl<R: rand::Rng> Image {
    pub fn draw_custom(&mut self, instruction: DrawInstruction<R>, rng: &mut R) {
        let mut new_layer = vec![coloring::TransparentColor::TRANSPARENT; self.canvas.len()];
        
        for y in 0..self.canvas_height() {
            for x in 0..self.canvas_width {
                let point = shapes::Point {x: x as f64, y: y as f64};

                new_layer[self.get_index(x, y)] = instruction.coloring.sample_color(&point);
            }
        }

        if let Some(noise) = instruction.pre_clip_noise {
            noise.add_noise(new_layer, rng);
        }
        
        for y  in 0..self.canvas_height() {
            for x in 0..self.canvas_width {
                let point = shapes::Point {x: x as f64, y: y as f64};
                
                // TODO antialiasing
                if !instruction.clipping_shape.contains(&point){
                    new_layer[self.get_index(x, y)] = TransparentColor::TRANSPARENT;
                }
            }
        }


        if let Some(noise) = instruction.post_clip_noise {
            noise.add_noise(self, rng);
        }

        for (index, canvas_color) in self.canvas.iter_mut().enumerate() {
            *canvas_color = new_layer[index].draw_on_solid(canvas_color);
        }
        

        if let Some(noise) = instruction.post_draw_noise {
            noise.add_noise(self, rng);
        }
        
    }

}
