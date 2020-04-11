use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Line,
    style::PrimitiveStyleBuilder
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
    SimulatorEvent
};
use nalgebra::{ Vector3, Matrix2x3, Rotation3 };
use std::{ thread, time::Duration };

static SCALE: f32 = 60.0;
static DISP_SIZE: f32 = 128.0;
static CENTER: f32 = DISP_SIZE / 2.0;

fn calculate_vertices<'a>(points: &'a Vec<Vector3<f32>>, projection: &'a Matrix2x3::<f32>, angle: &'a f32) -> Vec<(i32, i32)> {
    let rotation_y = Rotation3::new(Vector3::y() * *angle);
    let rotation_x = Rotation3::new(Vector3::x() * *angle);
    let rotation_z = Rotation3::new(Vector3::z() * *angle);

    points.iter().map(move |point| {
        let mut rotated = rotation_x * point;
        rotated = rotation_x * rotated;
        rotated = rotation_z * rotated;
        rotated = rotation_y * rotated;

        let mut projected_2d = projection * rotated;
        // TODO remove mutation and scale when x and y
        projected_2d.scale_mut(SCALE as f32);


        (
            (projected_2d.data[0] + CENTER) as i32,
            (projected_2d.data[1] + CENTER) as i32
        )
    }).collect::<Vec<(i32, i32)>>()
}

fn lines<'a>(vertices: &'a Vec<(i32,i32)>) -> impl Iterator<Item = Pixel<Rgb565>> + 'a {
    let decoration_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::RED)
        .stroke_width(1)
        .build();

    (0..=3).into_iter().map(move |i| {
        Line::new(
            Point::new(vertices[i].0, vertices[i].1),
            Point::new(vertices[(i+1) % 4].0,vertices[(i+1) % 4].1)
        )
        .into_styled(decoration_style)
        .into_iter()
        .chain(
            Line::new(
                Point::new(vertices[i+4].0, vertices[i+4].1),
                Point::new(vertices[((i+1) % 4) + 4].0, vertices[((i+1) % 4) + 4].1)
            )
            .into_styled(decoration_style)
            .into_iter()
        ).chain(
            Line::new(
                Point::new(vertices[i].0, vertices[i].1),
                Point::new(vertices[i+4].0, vertices[i+4].1)
            )
            .into_styled(decoration_style)
            .into_iter()
        )
    }).flatten()
}

fn main() -> Result<(), std::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(DISP_SIZE as u32, DISP_SIZE as u32));

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::Default)
        .build();

    let mut window = Window::new("3D cube", &output_settings);
    
    let points: Vec<Vector3<f32>> = vec![
        Vector3::new(-0.5, -0.5, -0.5),
        Vector3::new(0.5, -0.5, -0.5),
        Vector3::new(0.5, 0.5, -0.5),
        Vector3::new(-0.5, 0.5, -0.5),
        Vector3::new(-0.5, -0.5, 0.5),
        Vector3::new(0.5, -0.5, 0.5),
        Vector3::new(0.5, 0.5, 0.5),
        Vector3::new(-0.5, 0.5, 0.5)
    ];
    let projection = Matrix2x3::identity();

    let mut angle = 0.0;

    'running: loop {

        display.clear(Rgb565::BLACK)?;

        let vertices = calculate_vertices(&points, &projection, &angle);
        
        lines(&vertices).draw(&mut display)?;
        
        vertices.iter().map(|edge| {
            Pixel(Point::new(edge.0, edge.1), Rgb565::WHITE)
        }).draw(&mut display)?;
            
        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit){
            break 'running Ok(());
        }

        angle += 0.05;

        thread::sleep(Duration::from_millis(50));
    }
}