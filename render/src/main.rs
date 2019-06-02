use crate::web_mercator::Projection;
use geo::prelude::*;
use geo_types::Rect;
use graphics::polygon;
use graphics::types::Color;
use graphics::{ellipse, line};
use graphics_buffer::*;
use std::time::{Duration, Instant};
mod nz_map;
mod sample;
mod try_bounding_rect;
mod web_mercator;

use crate::try_bounding_rect::TryBoundingRect;

fn build_mapper(
    proj: Projection,
    rect_image_bounds: (f64, f64),
    offset: (f64, f64),
    points_bounds: Rect<f64>,
) -> impl Fn(&(f64, f64)) -> (f64, f64) {
    let coord_bounds = points_bounds
        .try_bounding_rect()
        .unwrap()
        .map_coords(&|point| proj.coord_mapper(point))
        .try_bounding_rect()
        .unwrap();
    println!("{:?}", coord_bounds);
    let (img_w, img_h) = rect_image_bounds;
    let (coord_w, coord_h) = (coord_bounds.width().abs(), coord_bounds.height().abs());
    let (coord_min_x, coord_min_y) = coord_bounds.min.x_y();

    let scale = {
        let scale_x = img_w / coord_w;
        let scale_y = img_h / coord_h;
        if scale_x < scale_y {
            scale_x
        } else {
            scale_y
        }
    };

    assert!(coord_w * scale <= img_w, "Incorrect scale selected!");
    assert!(coord_h * scale <= img_h, "Incorrect scale selected!");

    println!("{:?}, {:?}", img_w, img_h);
    println!("{:?}, {:?}", coord_w, coord_h);

    move |point| {
        // *point
        let (x, y) = proj.coord_mapper(point);
        (
            offset.0 + ((x - coord_min_x) * scale),
            offset.1 + ((y - coord_min_y) * scale),
        )
    }
}

const WATER_COLOR: Color = [0.69, 0.86, 1., 1.];
const LAND_COLOR: Color = [0.86, 0.86, 0.86, 1.];

fn main() {
    let start = Instant::now();
    let image_size = (1000, 700);
    let image_bounds = (image_size.0 as f64, image_size.1 as f64);
    let border = 15.0;
    let useable_area_size = (
        (image_size.0 as f64) - (border * 2.),
        (image_size.1 as f64) - (border * 2.),
    );
    let data_points = sample::coordinates();
    let proj = Projection { zoom: 19. };
    let mapper = build_mapper(
        proj.clone(),
        useable_area_size,
        (border, border),
        data_points.bounding_rect().unwrap(),
    );

    let img_coords = data_points.map_coords(&mapper);

    println!("Before rendering start: {:?}", start.elapsed());

    // Create a new RenderBuffer
    let mut buffer = RenderBuffer::new(image_size.0, image_size.1);
    buffer.clear(WATER_COLOR);

    for poly in nz_map::nz_shapes() {
        let (exterior_line, interior_lines) = poly.into_inner();
        let exterior_line = exterior_line
            .map_coords(&mapper)
            .points_iter()
            .map(|p| [p.x(), p.y()])
            .collect::<Vec<_>>();
        if exterior_line
            .iter()
            .any(|p| 0. < p[0] && p[0] < image_bounds.0 && 0. < p[1] && p[1] < image_bounds.1)
        {
            polygon(LAND_COLOR, &exterior_line, IDENTITY, &mut buffer)
        }
    }
    println!("New Zealand Render: {:?}", start.elapsed());

    for l in img_coords.lines() {
        line(
            [1.0, 0.0, 0.0, 0.7],
            2.,
            [l.start.x, l.start.y, l.end.x, l.end.y],
            IDENTITY,
            &mut buffer,
        );
    }
    println!("Line: {:?}", start.elapsed());

    // Save the buffer
    buffer.save("circles.png").unwrap();
    println!("Output: {:?}", start.elapsed());
}
