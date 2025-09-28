
use std::marker::PhantomData;

use crate::{shapes::{CheckInside, Point, Rect}, Image};



pub trait Noise<R: rand::Rng>{
    fn add_noise(&self, image: &mut Image, rng: &mut R);
}

pub trait PointSampler<R: rand::Rng>{
    fn sample(rng: &mut R) -> Point;
}

pub struct NoiseTypes<R: rand::Rng,N: PointSampler<R>> {
    sampler: N,
    noising_behavior: NoisingBehavior,
    _marker: PhantomData<R>,
}

impl<R: rand::Rng, N: PointSampler<R>> Noise<R> for NoiseTypes<R, N> {
    fn add_noise(&self, image: &mut Image, rng: &mut R) {
        self.inner_add_noise(image, rng);
    }
}

enum NoisingBehavior {
    BoundedNoise(BoundedNoise),
}

impl<R, N> NoiseTypes<R, N> {
    fn inner_add_noise(&self, image: &mut Image, rng: &mut R)  {
        let mut sample_point = ||self.sampler.sample(rng);
        match &self.noising_behavior {
            NoisingBehavior::BoundedNoise(bounded_noise) => bounded_noise.add_noise(image, &mut self.sampler),
        }       
    }
}

pub struct BoundedNoise {
    bounds: Rect,    
    swap_density: f64,
}

impl BoundedNoise {
    fn add_noise(&self, image: &mut Image, sample_point: &mut dyn FnMut() -> Point) {
        
        let total_iters = image.canvas_width as f64 * image.canvas_height() as f64 * self.swap_density;
        
        for _ in 0..(total_iters as usize){
            let point1 = self.sample_bounded_point(sample_point);
            let point2 = self.sample_bounded_point(sample_point);
            
            image.swap_pixels(point1.x as usize, point1.y as usize, point2.x as usize, point2.y as usize);
        }
    }
    
    fn sample_bounded_point(&self, sample_point: &mut dyn FnMut() -> Point) -> Point {
        const MAX_RETRIES: usize = 200;
        
        let max_bound_point = self.bounds.max_point();
        let random_point = sample_point();
        for _ in 0..MAX_RETRIES {
            if self.bounds.contains(&random_point) && random_point.x != max_bound_point.x && random_point.y != max_bound_point.y {
                return random_point;
            }
        }
        return random_point;
    }
}

impl<D: rand_distr::Distribution<f64>, R: rand::Rng> BoundedNoise {
    fn new(distr: D, bounds: Rect, swap_density: f64) -> NoiseTypes<R> {

        NoiseTypes {
            sample_point: Box::new(move |r: &mut R| Point {
                x: distr.sample(r),
                y: distr.sample(r),
            }),
            noising_behavior: NoisingBehavior::BoundedNoise(BoundedNoise { 
                bounds,
                swap_density,
            }),
        }
    }
}