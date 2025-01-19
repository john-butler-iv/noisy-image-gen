use std::collections::HashMap;
use std::io::BufRead;
use core::iter::Iterator;
use std::fmt::{self, Display};


pub trait Canvas<ColorMode> {
    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn get_pixel_index(&self, point: CanvasPoint) -> usize;
    /// Gets pixel at the coordinate. Panics if out of bounds
    fn get_pixel(&mut self, point: CanvasPoint) -> &ColorMode;
    /// draws the color over the pixel at the coordinate given 
    fn draw_pixel(&mut self, point: CanvasPoint, color: Color);
}

fn rect_point_iter(top_left: CanvasPoint, bottom_right: CanvasPoint) -> PointsIter {
    PointsIter {
        curr_x: top_left.x,
        curr_y: top_left.y,
        height: bottom_right.y - top_left.y,
        width: bottom_right.x - top_left.x,
    }
}

pub fn points<Canv:Canvas<ColorMode>, ColorMode>(canvas: &Canv) -> PointsIter {
    rect_point_iter(CanvasPoint{x: 0, y:0}, CanvasPoint {x: canvas.width(), y: canvas.height()})
}

pub struct OpaqueCanvas {
    height: usize,
    width: usize,
    pixels: Vec<OpaqueColor>,
}

#[derive(Copy, Clone)]
pub struct OpaqueColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl Into<Color> for OpaqueColor {
    fn into(self) -> Color {
        Color {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: MAX_ALPHA,
        }
    }
}

impl OpaqueCanvas {
    pub fn new(height: usize, width: usize, background_color: OpaqueColor) -> Self {
        OpaqueCanvas {
            height, width,
            pixels: vec![background_color; height * width],
        }
    }
}

impl Canvas<OpaqueColor> for OpaqueCanvas {
    fn height(&self) -> usize {
        self.height
    }
    fn width(&self) -> usize {
        self.width
    }

    fn get_pixel_index(&self, point: CanvasPoint) -> usize {
        (point.x + point.y * self.width) as usize
    }

    fn get_pixel(&mut self, point: CanvasPoint) -> &OpaqueColor {
        if point.x >= self.width() || point.y >= self.height() { return &OPAQUE_BLACK; }
        &self.pixels[self.get_pixel_index(point)]
    }

    /// draws the color over the pixel at the coordinate given 
    fn draw_pixel(&mut self, point: CanvasPoint, color: Color) {
        if point.x >= self.width() || point.y >= self.height() { return; }

        let index = self.get_pixel_index(point);
        let current_pixel = self.pixels[index];

        self.pixels[index] = color.draw_over_opaque(current_pixel);
    }
}

struct TransparentCanvas {
    height: usize,
    width: usize,
    pixels: Vec<Color>,
}

#[derive(Copy, Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}


pub struct PointsIter {
    curr_x: usize,
    curr_y: usize,
    height: usize,
    width: usize,
}

impl Iterator for PointsIter {
    type Item = CanvasPoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_y >= self.height {
            return None;
        }
        let point = CanvasPoint{
            x: self.curr_x,
            y: self.curr_y,
        };

        self.curr_x += 1;
        if self.curr_x >= self.width {
            self.curr_x = 0;
            self.curr_y += 1;
        }

        Some(point)
    }
}

impl Canvas<Color> for TransparentCanvas {
    fn height(&self) -> usize{
        self.height
    }
    fn width(&self) -> usize {
        self.width
    }

    fn get_pixel_index(&self, point: CanvasPoint) -> usize {
        (point.x + point.y * self.width) as usize
    }

    /// Gets pixel at the coordinate. Panics if out of bounds
    fn get_pixel(&mut self, point: CanvasPoint) -> &Color {
        if point.x >= self.width() || point.y >= self.height() { return &TRANSPARENT; }
        &self.pixels[self.get_pixel_index(point)]
    }

    /// draws the color over the pixel at the coordinate given 
    fn draw_pixel(&mut self, point: CanvasPoint, color: Color) {
        let index = self.get_pixel_index(point);
        let current_pixel = self.pixels[index];

        self.pixels[index] = color.draw_over(current_pixel);
    }
}

impl TransparentCanvas {
    pub fn new(height: usize, width: usize, background_color: Color) -> Self {
        TransparentCanvas {
            height, width,
            pixels: vec![background_color; height * width],
        }
    }
    pub fn copy_size<Canv, ColorMode> (canvas_from: &Canv, background_color: Color) -> Self where Canv: Canvas<ColorMode>{
        Self::new( canvas_from.height(), canvas_from.width(), background_color)
    }

}

const MAX_ALPHA:u8 = 0xFF;

const TRANSPARENT: Color = Color {
    red: 0,
    green: 0,
    blue: 0, 
    alpha: 0,
};

const OPAQUE_BLACK: OpaqueColor = OpaqueColor {
    red: 0,
    green: 0,
    blue: 0,
};


fn blend_hex_value(v1: u8, v2: u8, alpha: u8) -> u8 {
    let v1 = v1 as u16;
    let v2 = v2 as u16;
    let alpha = alpha as u16;
    ((v1 * alpha + v2 * (MAX_ALPHA as u16 - alpha)) / MAX_ALPHA as u16)as u8
}

impl Color {
    /// draws this color over another. 
    pub fn draw_over_opaque(self, color_under: OpaqueColor) -> OpaqueColor {
        OpaqueColor {
            red: blend_hex_value(self.red, color_under.red, self.alpha),
            green: blend_hex_value(self.green, color_under.green, self.alpha),
            blue: blend_hex_value(self.blue, color_under.blue, self.alpha),
        }
    }
    pub fn draw_over(self, color_under: Color) -> Color {
        // To figure out what our new color value should be we want the following to be true:
        // if C0 is some opaque color value (your red, green, or blue value), then if you
        // apply some other color value C1 with an alpha value A1 (from 0.0 to 1.0) and then
        // apply yet another color value C2 with an alpha value A2 (from 0.0 to 1.0), then
        // it should give the same value as if you took C0 and applied some other color value
        // C3 with an alpha of A3. We want to output that new color
        // 
        // C0 < C1, A1 < C2, A2 = C0 < C3, A3
        // C0 * (1-A1) + C1 * A1 < C2, A2 = C0 < C3, A3
        // (C0 * (1-A1) + C1 * A1) * (1-A2) + C2 * A2 = C0 < C3, A3
        // (C0 * (1-A1) + C1 * A1) * (1-A2) + C2 * A2 = C0 * (1 - A3) + C3 * A3
        // C0 * (1-A1) * (1-A2) + C1 * A1* (1-A2) + C2 * A2 = C0 * (1 - A3) + C3 * A3
        // C0 * [(1-A1) * (1-A2)] + C1 * A1* (1-A2) + C2 * A2 = C0 * (1 - A3) + C3 * A3
        // => 1 - A3 = (1 - A1) * (1 - A2)
        // => C1 * A1 * (1 - A2) + C2 * A2 = C3 * A3
        //
        // let's start with figuring out the alpha value:
        // => 1 - A3 = (1 - A1) * (1 - A2)
        //    A3 = 1 - (1 - A1) * (1 - A2)
        // so our alpha value is independent of the specific color we pick. Good!
        //
        // plugging that into the C3 formula we have,
        // => C3 * A3 = C1 * A1 * (1 - A2) + C2 * A2
        //    C3 = (C1 * A1 * (1 - A2) + C2 * A2) / A3
        //
        // now, let's convert so that our A3 ranges from 0 -> 255 instead of 0.0 -> 1.0
        // and in particular, we're going to try to rearrange so that we only divide by 255
        // for large numerators to avoid rounding errors
        // => A3 = 1 - (1 - A1) * (1 - A2)
        //    (A3/255) = 1 - (1 - (A1/255)) * (1 - (A2/255))
        //    A3 = 255 - 255 * (1 - (A1/255)) * (1 - (A2/255))
        //    A3 = 255 - 255 * (255 / 255 - (A1/255)) * (255 / 255 - (A2/255))
        //    A3 = 255 - 255 * ((255 - A1) / 255) * ((255 - A2) / 255))
        //    A3 = 255 - (255 - A1) * (255 - A2) / 255
        //
        // and C3:
        // => C3 = (C1 * (A1 / 255) * (1 - (A2 / 255)) + C2 * (A2 / 255)) / (A3 / 255)
        //    C3 = 255 * (C1 * (A1 / 255) * (1 - (A2 / 255)) + C2 * (A2 / 255)) / A3 
        //    C3 = (255 * C1 * (A1 / 255) * (1 - (A2 / 255)) + 255 * C2 * (A2 / 255)) / A3 
        //    C3 = (C1 * A1 * (1 - (A2 / 255)) + C2 * A2) / A3 
        //    C3 = (C1 * A1 * (255 / 255 - (A2 / 255)) + C2 * A2) / A3 
        //    C3 = (C1 * A1 * (255 - A2) / 255 + C2 * A2) / A3 
        //
        //  if you notice, the biggest number we can possibly get before dividing is
        //  C1 * A1 * (255 - A2), which is just three arbitary 8 bit numbers, so their product
        //  is going to be is going to be at most 255^3 = 16,581,375, which requires 24 bits.
        //  so we're going to have to do our computations in variables of at least that size.
        //  The best smallest type in Rust is u32.


        // TODO is this associative?

        let combined_alpha = MAX_ALPHA as u32 - (MAX_ALPHA - color_under.alpha) as u32 * (MAX_ALPHA - self.alpha) as u32 / MAX_ALPHA as u32;

        let combine_color = |color1: u32, alpha1: u32, color2: u32, alpha2: u32| -> u32 {
            (color1 * alpha1 * (MAX_ALPHA as u32 - alpha2) / MAX_ALPHA as u32 + color2 * alpha2) / combined_alpha
        };

        Color{
            red: combine_color(color_under.red as u32, color_under.alpha as u32, self.red as u32, self.alpha as u32) as u8,
            green: combine_color(color_under.green as u32, color_under.alpha as u32, self.green as u32, self.alpha as u32) as u8,
            blue: combine_color(color_under.blue as u32, color_under.alpha as u32, self.blue as u32, self.alpha as u32) as u8,
            alpha: combined_alpha as u8,
        }
    }
}


struct Draw {
    point_mask: Box<dyn PointMask>,
    coloring: Box<dyn Coloring>,
    noise: Box<dyn Noise>
}

#[derive(Copy, Clone)]
pub struct Point<I> {
    pub x: I,
    pub y: I,
}
pub type CanvasPoint = Point<usize>;

impl<I,J,K,L> Point<I> where I: Ord + std::ops::Sub<I, Output=J> + Clone, J: std::ops::Mul<J, Output=K> + Clone, K: std::ops::Add<K,Output=L> {
    fn square_dist_to(&self, other_point: &Point<I>) -> L {
        self.square_dist_to_literal(&other_point.x, &other_point.y)
    }
    fn square_dist_to_literal(&self, other_x: &I, other_y: &I) -> L {
        let dx = std::cmp::max(self.x.clone(), other_x.clone()) - std::cmp::min(self.x.clone(), other_x.clone());
        let dy = std::cmp::max(self.y.clone(), other_y.clone()) - std::cmp::min(self.y.clone(), other_y.clone());

        dx.clone() * dx + dy.clone() * dy
    }
}

pub trait PointMask {
    fn get_bounding_box(&self) -> (CanvasPoint, CanvasPoint);
    fn is_point_in_shape(&self, point: CanvasPoint) -> bool;
}

impl dyn PointMask {
    pub fn points(&self) -> PointsIter {
        let (point1, point2) = self.get_bounding_box();
        rect_point_iter(CanvasPoint {
            x: std::cmp::min(point1.x, point2.x), 
            y: std::cmp::min(point1.y, point2.y),
        }, 
        CanvasPoint {
            x: std::cmp::max(point1.x, point2.x), 
            y: std::cmp::max(point1.y, point2.y),
        })
    }
}

struct Rectangle {
    top_left: CanvasPoint,
    bottom_right: CanvasPoint,
}

impl Rectangle {
    pub fn new(point1: CanvasPoint, point2: CanvasPoint) -> Self{
        Rectangle {
            top_left: CanvasPoint {
                x: std::cmp::min(point1.x, point2.x),
                y: std::cmp::min(point1.y, point2.y),
            }, 
            bottom_right: CanvasPoint {
                x: std::cmp::max(point1.x, point2.x),
                y: std::cmp::max(point1.y, point2.y),
            }
        }
    }
}

impl PointMask for Rectangle {
    fn get_bounding_box(&self) -> (CanvasPoint, CanvasPoint) {
        (self.top_left, self.bottom_right)
    }

    fn is_point_in_shape(&self, point: CanvasPoint) -> bool {
        self.top_left.x <= point.x && point.x <= self.bottom_right.x
        && self.top_left.y <= point.y && point.y <= self.bottom_right.y
    }
}

struct Circle {
    center: Point<isize>,
    radius: usize,
}

impl Circle {
    pub fn new(center_x: isize, center_y: isize, radius: usize) -> Self {
        Circle {
            center: Point {
                x: center_x,
                y: center_y,
            },
            radius,
        }
    }
}

impl PointMask for Circle {
    fn get_bounding_box(&self) -> (CanvasPoint, CanvasPoint) {
        (
            CanvasPoint {
                x: usize::try_from(self.center.x - self.radius as isize).unwrap_or(0),
                y: usize::try_from(self.center.y - self.radius as isize).unwrap_or(0),
            },
            CanvasPoint {
                x: usize::try_from(self.center.x + self.radius as isize).unwrap_or(0),
                y: usize::try_from(self.center.y + self.radius as isize).unwrap_or(0),
            }
        )
    }

    fn is_point_in_shape(&self, point: CanvasPoint) -> bool {
        self.center.square_dist_to_literal(&(point.x as isize), &(point.y as isize)) < (self.radius * self.radius) as isize
    }
}

trait Coloring {
    fn get_color(&self, point: CanvasPoint) -> Color;
}

struct LinearGradient {
    pole1: GradientBase,
    pole2: GradientBase,
}

struct GradientBase {
    point: Point<isize>,
    color: Color,
}

impl Coloring for LinearGradient {
    fn get_color(&self, point: CanvasPoint) -> Color {
        let point = Point {
            x: point.x as isize,
            y: point.y as isize,
        };
        // (point - pole1 (dot) pole2 - pole1) 
        let dx = self.pole2.point.x - self.pole1.point.x;
        let dy = self.pole2.point.y - self.pole1.point.y;
        let relative_projected_point = Point {
            x: (point.x  - self.pole1.point.x ) * dx,
            y: (point.y  - self.pole1.point.y ) * dy,
        };

        let (red, green, blue, alpha) = 
        if relative_projected_point.x > relative_projected_point.y {
            (
                (relative_projected_point.x * self.pole1.color.red as isize 
                     + (dx - relative_projected_point.x) * self.pole2.color.red as isize) / dx,
                (relative_projected_point.x * self.pole1.color.green as isize 
                     + (dx - relative_projected_point.x) * self.pole2.color.green as isize) / dx,
                (relative_projected_point.x * self.pole1.color.blue as isize 
                     + (dx - relative_projected_point.x) * self.pole2.color.blue as isize) / dx,
                (relative_projected_point.x * self.pole1.color.alpha as isize 
                     + (dx - relative_projected_point.x) * self.pole2.color.alpha as isize) / dx,
            )
        } else {
            (
                (relative_projected_point.y * self.pole1.color.red as isize 
                     + (dy - relative_projected_point.y) * self.pole2.color.red as isize) / dy,
                (relative_projected_point.y * self.pole1.color.green as isize 
                     + (dy - relative_projected_point.y) * self.pole2.color.green as isize) / dy,
                (relative_projected_point.y * self.pole1.color.blue as isize 
                     + (dy - relative_projected_point.y) * self.pole2.color.blue as isize) / dy,
                (relative_projected_point.y * self.pole1.color.alpha as isize 
                     + (dy - relative_projected_point.y) * self.pole2.color.alpha as isize) / dy,
            )
        };

        Color {
            red: std::cmp::min(std::cmp::max(red, 0), 255) as u8,
            green: std::cmp::min(std::cmp::max(green, 0), 255) as u8,
            blue: std::cmp::min(std::cmp::max(blue, 0), 255) as u8,
            alpha: std::cmp::min(std::cmp::max(alpha, 0), 255) as u8,
        }
    }
}

struct LinearSampling {
    gradient_bases: Vec<GradientBase>,
}

impl Coloring for LinearSampling {
    fn get_color(&self, point: CanvasPoint) -> Color {
        let mut max_dist = 0;
        for gradient_base in self.gradient_bases.iter() {
            let dist = gradient_base.point.square_dist_to_literal(&(point.x as isize),&(point.y as isize)) as usize;

            max_dist = std::cmp::max(max_dist, dist);
        }
        let max_dist = max_dist;


        let mut weighted_reds = 0usize;
        let mut weighted_greens = 0usize;
        let mut weighted_blues = 0usize;
        let mut weighted_alphas = 0usize;


        for gradient_base in self.gradient_bases.iter() {
            let dist = gradient_base.point.square_dist_to_literal(&(point.x as isize),&(point.y as isize)) as usize;

            weighted_reds += gradient_base.color.red as usize * dist;
            weighted_greens += gradient_base.color.green as usize * dist;
            weighted_blues += gradient_base.color.blue as usize * dist;
            weighted_alphas += gradient_base.color.alpha as usize * dist;
        }



        Color {
            red: (weighted_reds / max_dist) as u8,
            green: (weighted_greens / max_dist) as u8,
            blue: (weighted_blues / max_dist) as u8,
            alpha: (weighted_alphas / max_dist) as u8,
        }
    }
}

trait Noise {
    fn apply_pre_clip(&self, canvas: &mut dyn Canvas<Color>);
    fn apply_post_merge(&self, canvas: &mut dyn Canvas<OpaqueColor>, point_mask: & dyn PointMask);
}



pub trait Drawable {
    fn draw_on(&self, canvas: &mut OpaqueCanvas);
}

impl Drawable for Draw {
    fn draw_on(&self, canvas: &mut OpaqueCanvas) {
        let mut sample_canvas = TransparentCanvas::copy_size(canvas, TRANSPARENT);
        for point in points(&sample_canvas) {
            sample_canvas.draw_pixel(point, self.coloring.get_color(point));
        }

        self.noise.apply_pre_clip(&mut sample_canvas);

        for point in self.point_mask.points() {
            canvas.draw_pixel(point, *sample_canvas.get_pixel(point));
        }

        self.noise.apply_post_merge(canvas, self.point_mask.as_ref());

    }
}


pub enum ReadFileError {
    IOError(std::io::Error),
    SyntaxError(String),
}

impl From<std::io::Error> for ReadFileError {
    fn from(error: std::io::Error) -> Self {
        ReadFileError::IOError(error)
    }
}

pub enum RValue {
    Literal(Literal),
    ConstName(String),
    Math(MathExpression),
}

impl RValue {
    fn try_from_helper(raw_string: &str, 
        symbol1: &str, constructor1: impl FnOnce(Box<RValue>, Box<RValue>)-> RValue,
        symbol2: &str, constructor2: impl FnOnce(Box<RValue>, Box<RValue>)-> RValue
    ) -> Result<Option<RValue>,ReadFileError> {

        let index1 = raw_string.find(symbol1);
        let index2 = raw_string.find(symbol2);

        if index1 == Some(0) || index2 == Some(0) || index1 == Some(raw_string.len() - 1) || index2 == Some(raw_string.len() - 1) {
            return Ok(None);
        }


        if let Some(index1) = index1 {
            if let Some(index2) = index2 {
                if index1 < index2{
                    let lhs = Box::new(RValue::try_from(&raw_string[..index1])?);
                    let rhs = Box::new(RValue::try_from(&raw_string[index1+symbol1.len()..])?);
                    Ok(Some(constructor1(lhs,rhs)))
                } else {
                    let lhs = Box::new(RValue::try_from(&raw_string[..index2])?);
                    let rhs = Box::new(RValue::try_from(&raw_string[index2+symbol2.len()..])?);
                    Ok(Some(constructor2(lhs,rhs)))
                }
            } else {
                    let lhs = Box::new(RValue::try_from(&raw_string[..index1])?);
                    let rhs = Box::new(RValue::try_from(&raw_string[index1+symbol1.len()..])?);
                    Ok(Some(constructor1(lhs,rhs)))
            }
        } else if let Some(index2) = index2 {
            let lhs = Box::new(RValue::try_from(&raw_string[..index2])?);
            let rhs = Box::new(RValue::try_from(&raw_string[index2+symbol2.len()..])?);
             Ok(Some(constructor2(lhs,rhs)))
        }
        else {
            Ok(None)
        }
    }

}

impl TryFrom<&str> for RValue {
    type Error = ReadFileError;

    fn try_from(raw_string: &str) -> Result<RValue, ReadFileError> {
        let raw_string = raw_string.trim();

        let mut lhs: Option<RValue> = None;

        if raw_string.chars().all(char::is_alphabetic) {
            return Ok(RValue::ConstName(raw_string.into()));
        }

        if let Ok(int_val) = raw_string.parse::<isize>() {
            return Ok(RValue::Literal(Literal::Integer(int_val)));
        }

        if raw_string.starts_with("(") {
            let mut depth = 1;
            let mut ending_index:Option<usize> = None;
            for (index, char) in raw_string.chars().enumerate().skip(1) {
                if char == '(' {
                    depth += 1;
                } else if char == ')' {
                    depth -= 1;
                }
                if depth == 0 {
                    ending_index = Some(index);
                    break;
                }
            }

            if let Some(ending_index) = ending_index {
                let lhs = RValue::try_from(&raw_string[1..ending_index])?;
                let rest = raw_string[ending_index + 1..].trim();
                
                match rest.chars().next() {
                    None => return Ok(lhs),
                    Some('*') => return Ok(RValue::Math(MathExpression::Multiply(Box::new(lhs), Box::new(RValue::try_from(&raw_string[1..])?)))),
                    Some('/') => return Ok(RValue::Math(MathExpression::Divide(Box::new(lhs), Box::new(RValue::try_from(&raw_string[1..])?)))),
                    Some('+') => return Ok(RValue::Math(MathExpression::Add(Box::new(lhs), Box::new(RValue::try_from(&raw_string[1..])?)))),
                    Some('-') => return Ok(RValue::Math(MathExpression::Subtract(Box::new(lhs), Box::new(RValue::try_from(&raw_string[1..])?)))),
                    _ => return Err(ReadFileError::SyntaxError("Invalid operation performed to the right of parentheses".into()))
                }
            } else {
                return Err(ReadFileError::SyntaxError("Unmatched parentheses".into()));
            }
        }

        let create_multiply = |lhs: Box<RValue>, rhs: Box<RValue>|RValue::Math(MathExpression::Multiply(lhs, rhs));
        let create_divide = |lhs: Box<RValue>, rhs: Box<RValue>|RValue::Math(MathExpression::Divide(lhs, rhs));
        if let Some(r_value) = RValue::try_from_helper(raw_string, "*", create_multiply, "/", create_divide)? {
            return Ok(r_value)
        }

        let create_add = |lhs: Box<RValue>, rhs: Box<RValue>|RValue::Math(MathExpression::Add(lhs, rhs));
        let create_subtract = |lhs: Box<RValue>, rhs: Box<RValue>|RValue::Math(MathExpression::Subtract(lhs, rhs));
        if let Some(r_value) = RValue::try_from_helper(raw_string, "+", create_add, "-", create_subtract)? {
            return Ok(r_value)
        }


        if raw_string.starts_with("#") {
            match raw_string.len(){
                4 => { // #rgb
                    let red = if let Ok(red) = u32::from_str_radix(raw_string[1], 16) {
                        red
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let green = if let Ok(green) = u32::from_str_radix(raw_string[2], 16) {
                        green
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let blue = if let Ok(blue) = u32::from_str_radix(raw_string[3], 16) {
                        blue
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    return Ok(RValue::Literal(Literal::Color(OpaqueColor {red, green, blue}.into())));
                },
                5 => { // #rgba
                    let red = if let Ok(red) = u32::from_str_radix(raw_string[1], 16) {
                        red
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let green = if let Ok(green) = u32::from_str_radix(raw_string[2], 16) {
                        green
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let blue = if let Ok(blue) = u32::from_str_radix(raw_string[3], 16) {
                        blue
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let alpha = if let Ok(alpha) = u32::from_str_radix(raw_string[3], 16) {
                        alpha
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    return Ok(RValue::Literal(Literal::Color(Color {red, green, blue, alpha})));
                },
                7 => { // #rrggbb
                    let red = if let Ok(red) = u32::from_str_radix(&raw_string[1..3], 16) {
                        red
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let green = if let Ok(green) = u32::from_str_radix(&raw_string[3..5], 16) {
                        green
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let blue = if let Ok(blue) = u32::from_str_radix(&raw_string[5..7], 16) {
                        blue
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    return Ok(RValue::Literal(Literal::Color(OpaqueColor {red, green, blue}.into())));
                },
                9 => { // ##rrbbggaa
                    let red = if let Ok(red) = u32::from_str_radix(&raw_string[1..3], 16) {
                        red
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let green = if let Ok(green) = u32::from_str_radix(&raw_string[3..5], 16) {
                        green
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let blue = if let Ok(blue) = u32::from_str_radix(&raw_string[5..7], 16) {
                        blue
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    let alpha = if let Ok(alpha) = u32::from_str_radix(&raw_string[7..9], 16) {
                        alpha
                    } else {
                        return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}")));
                    };
                    return Ok(RValue::Literal(Literal::Color(Color {red, green, blue, alpha})));
                },
                _ => return Err(ReadFileError::SyntaxError(format!("Invalid color hex code {raw_string}"))),
            }
        }

        Err(ReadFileError::SyntaxError(format!("Invalid expression {raw_string}")))
    }
}

impl Display for RValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "");
        todo!()
    }
}

pub enum Literal {
    Color(Color),
    Integer(isize),
}

pub enum MathExpression {
    Multiply(Box<RValue>,Box<RValue>),
    Divide(Box<RValue>,Box<RValue>),
    Add(Box<RValue>,Box<RValue>),
    Subtract(Box<RValue>, Box<RValue>),
}

pub struct Instruction {
    label: String,
    properties: HashMap<String, RValue>,
}

pub fn read_file<P>(noisy_filename: P) -> Result<(OpaqueCanvas, Vec<Box<dyn Drawable>>), ReadFileError> 
where P: AsRef<std::path::Path> {
    let mut const_table: HashMap<&str, RValue> = HashMap::new();
    let mut instruction_list: Vec<Instruction> = Vec::new();
    let mut current_instruction: Option<Instruction> = None;

    for (line_num, line) in std::io::BufReader::new(std::fs::File::open(noisy_filename)?).lines().enumerate() {
        let line: String = line?.split("//").next().unwrap().trim().to_lowercase().into();
        if line.is_empty() {
            continue;

        } else if line.starts_with("#const ") {
            let mut pieces = line[7..].split("=");
            let label = pieces.next().unwrap().trim().to_owned();
            if label.is_empty() {
                return Err(ReadFileError::SyntaxError(
                        format!("Invalid #const definition on line {line_num}. You must pick a name to reference the constant value with.")
                ));
            }
            if const_table.contains_key(label.as_ref()) {
                return Err(ReadFileError::SyntaxError(
                        format!("Invalid #const definition for on line {line_num}. {label} is already used for another variable with value {}. Capitalization is ignored.", const_table.get(label).unwrap())
                ));
            }
            let value = if let Some(value) = pieces.next(){
                RValue::try_from(value.trim())
            } else {
                Err(ReadFileError::SyntaxError(
                        format!("Invalid #const definition on line {line_num}. You must set a value.")
                ))
            }?;
            const_table.insert(&label, value);
        } else if let Some(instruction) = current_instruction.take() {
            if line == "}" {
                instruction_list.push(instruction);
            } else if line.contains("}") {
                return Err(ReadFileError::SyntaxError("Block closing braces must be on their own lines.".into()));
            } else if line.contains("{") {
                return Err(ReadFileError::SyntaxError("Blocks cannot contain other blocks.".into()));
            } else {

                // TODO otherwise record property (first label) and RValue

                // TODO detect duplicate properties

                current_instruction = Some(instruction);
            }

        } else {
            if !line.ends_with("{"){
                return Err(ReadFileError::SyntaxError("All instructions outside of a block must either be a #const declaration or a write instruction followed by curly braces".into()));
            }
            let label = &line[..line.len()-1];
            if !label.chars().all(char::is_alphabetic) {
                return Err(ReadFileError::SyntaxError(format!("Invalid draw instruction {label}")));
            }

            current_instruction = Some(Instruction {
                label: label.into(),
                properties: HashMap::new(),
            });
        }
    }

    // TODO flatten consts (detect reference loops)
    // TODO create intermediate objects
    //
    // TODO decide on Noise object stuff

    todo!()
}


