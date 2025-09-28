use crate::shapes::Point;


pub trait Color: Sized + Copy {
    fn mix(color_weights: &[(Self, f64)]) -> Self;
}

#[derive(Copy, Clone, Debug)]
pub struct SolidColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Into<image::Rgb<u8>> for SolidColor{
    fn into(self) -> image::Rgb<u8> {
        image::Rgb::from([self.red, self.green, self.blue])
    }
}

impl Color for SolidColor {
    fn mix(color_weights: &[(Self, f64)]) -> Self {
        let transparent_weights: Vec<(TransparentColor, f64)> = color_weights.iter().map(|(solid_color, weight)| 
            (solid_color.clone().into(), *weight)
        ).collect();
        TransparentColor::mix(&transparent_weights).as_solid()
    }
}

impl SolidColor {
    pub const BLACK: SolidColor = SolidColor {
        red: 0,
        green: 0,
        blue: 0,
    };

    pub fn from_hex_code(hex_code: &str) -> SolidColor {
        let orig_hex_code = hex_code;
        let hex_code = if hex_code.chars().nth(0) == Some('#') {
            &hex_code[1..]
        } else { hex_code };

        if hex_code.len() == 8 && u8::from_str_radix(&hex_code[6..8], 16).is_err(){
            panic!("Invalid hex code \"{orig_hex_code}\"");
        }

        match TransparentColor::from_hex_code(orig_hex_code).try_into() {
            Ok(solid_color) => solid_color,
            Err(()) => panic!("alpha component specified for a solid color. Did you mean to call Color::from_hex_code(\"{orig_hex_code}\") instead?")
        }
    }
}

impl Into<TransparentColor> for SolidColor {
    fn into(self) -> TransparentColor {
        TransparentColor {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: u8::MAX,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TransparentColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Into<image::Rgba<u8>> for TransparentColor {
    fn into(self) -> image::Rgba<u8> {
        image::Rgba::from([self.red, self.green, self.blue, self.alpha])
    }
}

impl Color for TransparentColor {
    fn mix(color_weights: &[(Self, f64)]) -> Self {
        let mut running_red = 0.;
        let mut running_green = 0.;
        let mut running_blue = 0.;
        let mut running_alpha = 0.;
    
        for (color, weight) in color_weights {
            running_red += color.red as f64 * weight;
            running_green += color.green as f64 * weight;
            running_blue += color.blue as f64 * weight;
            running_alpha += color.alpha as f64 * weight;
        }
        
        running_red = running_red.clamp(0., 255.);
        running_green = running_green.clamp(0., 255.);
        running_blue = running_blue.clamp(0., 255.);
        running_alpha = running_alpha.clamp(0., 255.);
    
        TransparentColor {
            red: running_red as u8,
            green: running_green as u8,
            blue: running_blue as u8,
            alpha: running_alpha as u8,
        }
    }
}

impl TryInto<SolidColor> for TransparentColor {
    type Error = ();
    fn try_into(self) -> Result<SolidColor, Self::Error>{
        if self.alpha == u8::MAX {
            Ok(SolidColor {
                red: self.red,
                green: self.green,
                blue: self.blue,
            })
        } else {
            Err(())
        }
    }
}

impl TransparentColor {
    pub const TRANSPARENT: TransparentColor = TransparentColor {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    };

    pub fn from_hex_code(hex_code: &str) -> Self {
        let orig_hex_code = hex_code;
        let hex_code = if hex_code.chars().nth(0) == Some('#') {
            &hex_code[1..]
        } else { hex_code };

        let red: u8;
        let green: u8; 
        let blue: u8;
        let mut alpha: u8;
        if hex_code.len() == 6 || hex_code.len() == 8 {
            red = u8::from_str_radix(&hex_code[0..2], 16).expect(&format!("Invalid red component in hex code \"{orig_hex_code}\""));
            green = u8::from_str_radix(&hex_code[2..4], 16).expect(&format!("Invalid green component in hex code \"{orig_hex_code}\""));
            blue = u8::from_str_radix(&hex_code[4..6], 16).expect(&format!("Invalid blue component in hex code \"{orig_hex_code}\""));
            alpha = u8::MAX;
       } else {
           panic!("Invalid hex code {orig_hex_code}");
       }

        if hex_code.len() == 8 {
            alpha = u8::from_str_radix(&hex_code[4..6], 16).expect(&format!("Invalid alpha component in hex code \"{orig_hex_code}\""));
        }


        TransparentColor {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn as_solid(&self) -> SolidColor {
        SolidColor {
            red: self.red,
            green: self.green,
            blue: self.blue,
        }
    }

    pub fn draw_on_solid(&self, base_color: &SolidColor) -> SolidColor {
        let find_new_color = |color1: u8, color2: u8| -> u8{
            let color1 = color1 as u16;
            let color2 = color2 as u16;
            let alpha2 = self.alpha as u16;
            
            (color1 * (u8::MAX as u16 - alpha2) / u8::MAX as u16 + color2 * alpha2 / u8::MAX as u16) as u8
        };

        SolidColor {
            red: find_new_color(base_color.red, self.red),
            green: find_new_color(base_color.green, self.green),
            blue: find_new_color(base_color.blue, self.blue),
        }
    }


    pub fn draw_on(&self, base_color: &TransparentColor) -> TransparentColor {
        let new_alpha = self.alpha as u32 + base_color.alpha as u32 - (self.alpha as u32 * base_color.alpha as u32) / 255;
        let find_new_color = |color1: u8, color2: u8| -> u8{
            let color1 = color1 as u32;
            let color2 = color2 as u32;
            let alpha2 = self.alpha as u32;
            
            let numer = color1 * color2 * (u8::MAX as u32 - alpha2) + (u8::MAX as u32) * color2 * alpha2;
            (numer / new_alpha) as u8
        };

        TransparentColor {
            red: find_new_color(base_color.red, self.red),
            green: find_new_color(base_color.green, self.green),
            blue: find_new_color(base_color.blue, self.blue),
            alpha: new_alpha as u8,
        }
    }

}


pub trait Coloring {
    type ColorType; 
    fn sample_color(&self, point: &Point) -> Self::ColorType;
}

#[derive(Clone, Debug)]
pub enum ColorScheme<ColorType: Color> {
    LinearGradient(LinearGradient<ColorType>),
    ComplexGradient(ComplexGradient<ColorType>),
}

impl<ColorType: Color> Coloring for ColorScheme<ColorType> {
    type ColorType = ColorType;
    fn sample_color(&self, point: &Point) -> Self::ColorType {
        match self {
            ColorScheme::LinearGradient(grad) => grad.sample_color(point),
            ColorScheme::ComplexGradient(grad) => grad.sample_color(point),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LinearGradient<ColorType: Color> {
    pole1: (Point, ColorType),
    pole2: (Point, ColorType),
}

impl<ColorType: Color> Into<ColorScheme<ColorType>> for LinearGradient<ColorType> {
    fn into(self) -> ColorScheme<ColorType> {
        ColorScheme::LinearGradient(self)
    }
}

impl<ColorType: Color> LinearGradient<ColorType> {
    pub fn with_poles(pole1: (Point, ColorType), pole2: (Point, ColorType)) -> LinearGradient<ColorType> {
        if pole1.0.x == pole2.0.x {
            if pole1.0.y == pole2.0.y {
                panic!("Gradient poles must be distinct");
            } else if pole1.0.y < pole2.0.y {
                LinearGradient {
                    pole1, pole2
                }
            } else {
            LinearGradient {
                pole1: pole2,
                pole2: pole1,
            }
            }
        } else if pole1.0.x < pole2.0.x {
            LinearGradient {
                pole1, pole2
            }
        } else {
            LinearGradient {
                pole1: pole2,
                pole2: pole1,
            }
        }
    }
}

impl<ColorType: Color> Coloring for LinearGradient<ColorType> {
    type ColorType = ColorType;

    fn sample_color(&self, point: &Point) -> Self::ColorType {

        // if beyond the bounds of the gradient, just saturate to the closest point
        if self.pole1.0.x == self.pole2.0.x {
            if point.y < self.pole1.0.y {
                return self.pole1.1.clone();
            } 
            if point.y > self.pole2.0.y {
                return self.pole2.1.clone()
            }
        } else {
            if point.x < self.pole1.0.x {
                return self.pole1.1.clone();
            } 
            if point.x > self.pole2.0.x {
                return self.pole2.1.clone();
            }
        }

        let dist1 = point.dist_to(&self.pole1.0);
        let dist2 = point.dist_to(&self.pole2.0);

        let total_dist = dist1 + dist2;

        let portion1 = dist1 / total_dist;
        let portion2 = 1.0 - portion1;

        Self::ColorType::mix(&[(self.pole1.1, portion1), (self.pole2.1, portion2)])
    }
}

#[derive(Clone, Debug)]
pub struct ComplexGradient<ColorType: Color>{
    poles: Vec<(Point, ColorType)>,
}

impl<ColorType: Color> Into<ColorScheme<ColorType>> for ComplexGradient<ColorType> {
    fn into(self) -> ColorScheme<ColorType> {
        ColorScheme::ComplexGradient(self)
    }    
}

impl<ColorType: Color> ComplexGradient<ColorType> {
    pub const fn new() -> Self {
        ComplexGradient { 
            poles: Vec::new()
        }
    }

    pub fn add_pole(&mut self, location: Point, color: ColorType) {
        for (existing_pole,_) in self.poles.iter() {
            if *existing_pole == location {
                panic!("You cannot have two overlapping poles");
            }
        }
        self.poles.push((location, color));
    }
}

impl<ColorType: Color> Coloring for ComplexGradient<ColorType> {
    type ColorType = ColorType;
    fn sample_color(&self, point: &Point) -> Self::ColorType {
        let total_dist: f64 = self.poles.iter().map(|(pole, _)|point.dist_to(pole)).sum();
        let scaled_poles = 
            &self.poles.iter().map(|(pole, color)|{
                (*color, point.dist_to(pole) / total_dist)
            }).collect::<Vec<_>>();
        Self::ColorType::mix(scaled_poles)
    }
}

