use image_gen::{coloring::{LinearGradient, SolidColor }, shapes::{Area, Ellipse, Point, Rect}, DrawInstruction, Image};

fn main() {
    //image_gen::read_noisy_file("./Sample.noisy")
    let mut image = Image::with_size(2560,1440,SolidColor::BLACK);
    
    let origin = Point::ORIGIN;
    let far_corner = Point {x: 2560.0, y: 1440.0};
    let screen_area = Area::bounding_area(&origin, &far_corner);
    
    image.draw_custom(DrawInstruction{
        pre_clip_noise: None,
        clipping_shape: Rect::from_points(&origin, &far_corner).into(),
        coloring: LinearGradient::with_poles(
            (origin, SolidColor {red: 5, green: 47, blue: 95 }.into()), 
            (far_corner, SolidColor {red: 6, green: 167, blue: 125 }.into())
        ).into(),
        post_clip_noise: None,
    });
    
    let center = Point {
        x: (far_corner.x + origin.x) / 2.,
        y: (far_corner.y + origin.y) / 2.,
    };
    let radius = 3. * f64::min(screen_area.height, screen_area.width) / 8.;
    
    image.draw_custom(DrawInstruction { 
        pre_clip_noise: None,
        clipping_shape: Ellipse::circle(center, radius).into(),
        coloring: LinearGradient::with_poles(
            (origin, SolidColor {red: 186, green: 46, blue: 55}.into()), 
            (far_corner, SolidColor {red: 180, green: 121, blue: 6 }.into())
        ).into(),
        post_clip_noise: None,
    });
    
    let _ = image.output_to_image("./output.png");
}
