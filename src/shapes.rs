use std::ops::Div;


#[derive(Copy, Clone, Debug,  PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0., y: 0. };

    pub fn square_dist_to(&self, other: &Point) -> f64 {
        let x_diff = other.x - self.x;
        let y_diff = other.y - self.y;
        x_diff * x_diff + y_diff * y_diff
    }
    pub fn dist_to(&self, other: &Point) -> f64 {
        self.square_dist_to(other).sqrt()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Area {
    pub height: f64,
    pub width: f64,
}

impl Area {
    pub const EMPTY: Area = Area { height: 0., width: 0.};
    
    pub fn bounding_area(point1: &Point, point2: &Point) -> Self {
        let (min_x, max_x) = if point1.x <= point2.x {
            (point1.x, point2.x)
        } else {
            (point2.x, point1.x)
        };
        let (min_y, max_y) = if point1.y <= point2.y {
            (point1.y, point2.y)
        } else {
            (point2.y, point1.y)
        };
        
        Area {
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

pub trait CheckInside {
    fn contains(&self, point: &Point) -> bool;
}

pub enum Shape {
    Rect(Rect),
    Ellipse(Ellipse),
    TransformedShape(TransformedShape),
}

impl CheckInside for Shape {
    fn contains(&self, point: &Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(point),
            Shape::Ellipse(ellipse) => ellipse.contains(point),
            Shape::TransformedShape(trans_shape) => trans_shape.contains(point),
        }
    }
}


pub struct TransformedShape {
    inner_shape: Box<Shape>,
    transformation: Transformation,
}

impl Into<Shape> for TransformedShape {
    fn into(self) -> Shape {
        Shape::TransformedShape(self)
    }
}

impl CheckInside for TransformedShape {
    fn contains(&self, point: &Point) -> bool {
        self.inner_shape.as_ref().contains(&self.transformation.transform(point))
    }
}


pub trait Transform {
    fn transform(&self, point: &Point) -> Point;
    fn get_inverse(&self) -> Transformation;
    fn inverse_transform(&self, point: &Point) -> Point {
        self.get_inverse().transform(point)
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Transformation {
    Rotation(Rotation),
    Translation(Translation),
    Scale(Scale)
}
impl Transform for Transformation {
    fn transform(&self, point: &Point) -> Point{
        match self {
            Self::Rotation(rotation) => rotation.transform(point),
            Self::Translation(translation) => translation.transform(point),
            Self::Scale(scale) => scale.transform(point),
        }
    }

    fn get_inverse(&self) -> Self {
        match self {
            Self::Rotation(rotation) => rotation.get_inverse(),
            Self::Translation(translation) => translation.get_inverse(),
            Self::Scale(scale) => scale.get_inverse(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rotation {
    angle: f64,
    center_of_rotation: Translation,
}
impl Into<Transformation> for Rotation {
    fn into(self) -> Transformation {
        Transformation::Rotation(self)
    }
}

impl Rotation {
    pub const fn identity() -> Self {
        Self::rot_origin(0.)
    }

    pub const fn rot_origin(angle: f64) -> Self {
        Rotation{
            angle,
            center_of_rotation: Translation::identity()
        }
    }

    pub const fn rotate(angle: f64, center_of_rotation: Point) -> Self {
        Rotation{
            angle,
            center_of_rotation: Translation::to(center_of_rotation)
        }
    }
}


impl Transform for Rotation {
    fn transform(&self, point: &Point) -> Point {
        let rotatable_point = self.center_of_rotation.transform(point);

        let rotated_point = Point {
            x: f64::cos(self.angle) * rotatable_point.x,
            y: f64::sin(self.angle) * rotatable_point.y
        };

        self.center_of_rotation.inverse_transform(&rotated_point)
    }

    fn get_inverse(&self) -> Transformation {
        Rotation {
            angle: -self.angle,
            center_of_rotation: self.center_of_rotation
        }.into()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Translation {
    new_origin: Point,
}
impl Into<Transformation> for Translation{
    fn into(self) -> Transformation {
        Transformation::Translation(self)
    }
}

impl Translation {
    const fn identity() -> Self {
        Self::to(Point::ORIGIN)
    }

    const fn to(new_origin: Point) -> Self {
        Translation{
            new_origin
        }
    }
}

impl Transform for Translation {
    fn transform(&self, point: &Point) -> Point {
        Point {
            x: point.x + self.new_origin.x,
            y: point.y + self.new_origin.y,
        }
    }
    fn get_inverse(&self) -> Transformation {
        Translation {
            new_origin: Point {
                x: -self.new_origin.x,
                y: -self.new_origin.y,
            }
        }.into()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Scale {
    fixed_point: Translation,
    scalar: Area,
}

impl Scale {
    pub const fn identity() -> Self {
        Scale::by(Area::EMPTY)
    }

    pub const fn by(scalar: Area) -> Self {
        Scale::by_from(scalar, Point::ORIGIN)
    }

    pub const fn by_from(scalar: Area, from: Point) -> Self {
        Scale{
            fixed_point: Translation::to(from),
            scalar,
        }
    }
}

impl Into<Transformation> for Scale{
    fn into(self) -> Transformation {
        Transformation::Scale(self)
    }
}

impl Transform for Scale {
    fn transform(&self, point: &Point) -> Point {
        let scalable_point = self.fixed_point.transform(point);

        let scaled_point = Point {
            x: self.scalar.width * scalable_point.x,
            y: self.scalar.height * scalable_point.y
        };

        self.fixed_point.inverse_transform(&scaled_point)
    }

    fn get_inverse(&self) -> Transformation {
        Scale {
            fixed_point: self.fixed_point,
            scalar: Area {
                height: (1.0_f64).div(self.scalar.height),
                width: (1.0_f64).div(self.scalar.width),
            },
        }.into()
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Rect {
    min_point: Point,
    size: Area,
}

impl Into<Shape> for Rect {
    fn into(self) -> Shape {
        Shape::Rect(self)
    }
}

impl Rect {
    
    pub fn from_points(point1: &Point, point2: &Point) -> Self {
        let (min_x, max_x) = if point1.x <= point2.x {
            (point1.x, point2.x)
        } else {
            (point2.x, point1.x)
        };
        let (min_y, max_y) = if point1.y <= point2.y {
            (point1.y, point2.y)
        } else {
            (point2.y, point1.y)
        };
        
        Rect {
            min_point: Point {
                x: min_x,
                y: min_y,
            },
            size: Area {
                height: max_y - min_y,
                width: max_x - min_x,
            },
        }
    }

    pub fn max_point(&self) -> Point {
        Point {
            x: self.min_point.x + self.size.width,
            y: self.min_point.y + self.size.height,
        }
    }
}

impl CheckInside for Rect {
    fn contains(&self, point: &Point) -> bool {
        return point.x >= self.min_point.x 
            && point.y >= self.min_point.y 
            && point.x <= self.max_point().x 
            && point.y <= self.max_point().y
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Ellipse {
    center: Point,
    bounding_area: Area,
}

impl Into<Shape> for Ellipse {
    fn into(self) -> Shape {
        Shape::Ellipse(self)
    }
}

impl CheckInside for Ellipse {
    fn contains(&self, point: &Point) -> bool {
        let compute_part = |test_val: f64, center_val: f64, radius: f64| {
            let numer_sqrt = test_val - center_val;
            (numer_sqrt * numer_sqrt) / (radius * radius)
        };

        let x_part = compute_part(point.x, self.center.x, self.bounding_area.width / 2.);
        let y_part = compute_part(point.y, self.center.y, self.bounding_area.height / 2.);

        x_part + y_part <= 1.
    }
}

impl Ellipse {
    pub fn circle(center: Point, radius: f64) -> Self {
        Ellipse {
            center, 
            bounding_area: Area { height: radius * 2., width: radius * 2. } 
        }
    }
}
