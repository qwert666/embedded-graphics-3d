#![no_std]
#![no_main]

extern crate compiler_builtins;

use panic_halt as _;

use embedded_graphics::{
  pixelcolor::Rgb565,
  prelude::*,
  primitive_style,
  primitives::Line,
  style::PrimitiveStyleBuilder,
  primitives::Rectangle
};
use nalgebra::{ Vector3, Matrix2x3, Rotation3 };
use gd32vf103xx_hal::pac;
use gd32vf103xx_hal::prelude::*;
use longan_nano::{lcd, lcd_pins};
use riscv_rt::entry;
use heapless::Vec;
use heapless::consts::U8;


static SCALE: f64 = 40.0;
static DISP_SIZE: f64 = 128.0;

fn calculate_vertices<'a>(points: &'a Vec<Vector3<f64>, U8>, projection: &'a Matrix2x3::<f64>, angle: &'a f64, width: &'a i32, height: &'a i32) -> Vec<(i32, i32), U8> {
  let rotation_y = Rotation3::new(Vector3::y() * *angle as f64);
  let rotation_x = Rotation3::new(Vector3::x() * *angle as f64);
  let rotation_z = Rotation3::new(Vector3::z() * *angle as f64);

  points.iter().map(move |point| {
      let mut rotated = rotation_x * point;
      rotated = rotation_x * rotated;
      rotated = rotation_z * rotated;
      rotated = rotation_y * rotated;

      let mut projected_2d = projection * rotated;
      projected_2d.scale_mut(SCALE as f64);


      (
          (projected_2d.data[0] + *width as f64) as i32,
          (projected_2d.data[1] + *height as f64) as i32
      )
  }).collect::<Vec<(i32, i32), U8>>()
}

fn lines<'a>(vertices: &'a Vec<(i32,i32), U8>) -> impl Iterator<Item = Pixel<Rgb565>> + 'a {
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

#[entry]
fn main() -> ! {
  let dp = pac::Peripherals::take().unwrap();

  // Configure clocks
  let mut rcu = dp
    .RCU
    .configure()
    .ext_hf_clock(8.mhz())
    .sysclk(108.mhz())
    .freeze();
  let mut afio = dp.AFIO.constrain(&mut rcu);

  let gpioa = dp.GPIOA.split(&mut rcu);
  let gpiob = dp.GPIOB.split(&mut rcu);

  let lcd_pins = lcd_pins!(gpioa, gpiob);
  let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
  let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

  let mut points: Vec<Vector3<f64>, U8> = Vec::new();

  points.extend_from_slice(&[
    Vector3::new(-0.5, -0.5, -0.5),
    Vector3::new(0.5, -0.5, -0.5),
    Vector3::new(0.5, 0.5, -0.5),
    Vector3::new(-0.5, 0.5, -0.5),
    Vector3::new(-0.5, -0.5, 0.5),
    Vector3::new(0.5, -0.5, 0.5),
    Vector3::new(0.5, 0.5, 0.5),
    Vector3::new(-0.5, 0.5, 0.5)
  ]).unwrap();

  let projection = Matrix2x3::identity();

  let mut angle = 0.0;

  loop {

    // clear screen
    Rectangle::new(Point::new(0, 0), Point::new(width + 1, height + 1))
      .into_styled(primitive_style!(fill_color = Rgb565::BLACK))
      .draw(&mut lcd)
      .unwrap();

    let vertices = calculate_vertices(&points, &projection, &angle, &(&width / 2), &(&height / 2));
    
    lines(&vertices).draw(&mut lcd).unwrap();

    vertices.iter().map(|edge| {
      Pixel(Point::new(edge.0, edge.1), Rgb565::WHITE)
    }).draw(&mut lcd).unwrap();

    angle += 0.05;

  }
}
