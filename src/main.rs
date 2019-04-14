use graphics::ellipse;
use graphics_buffer::*;
mod sample;
mod web_mercator;

fn bounds(data: &[(f64, f64)]) -> Option<((f64, f64), (f64, f64))> {
    if data.is_empty() {
        return None;
    }
    let mut min_lon = data[0].0;
    let mut min_lat = data[0].1;
    let mut max_lon = min_lon;
    let mut max_lat = min_lat;

    for &(lon, lat) in data {
        if lon < min_lon {
            min_lon = lon;
        } else if lon > max_lon {
            max_lon = lon;
        }
        if lat < min_lat {
            min_lat = lat;
        } else if lat > max_lat {
            max_lat = lat;
        }
    }
    Some(((min_lon, min_lat), (max_lon, max_lat)))
}

fn main() {
    let image_size = 1000;
    let data = sample::COORDINATES;

    let ((min_lon, min_lat), (max_lon, max_lat)) = bounds(data).unwrap();
    let lon_diff = max_lon - min_lon;
    let lat_diff = max_lat - max_lon;

    // Create a new RenderBuffer
    let mut buffer = RenderBuffer::new(image_size, image_size);
    buffer.clear([0.0, 0.0, 0.0, 0.0]);

    // Big red circle
    ellipse(
        [1.0, 0.0, 0.0, 0.7],
        [0.0, 0.0, 100.0, 100.0],
        IDENTITY,
        &mut buffer,
    );
    // Small blue circle
    ellipse(
        [0.0, 0.0, 1.0, 0.7],
        [0.0, 0.0, 50.0, 50.0],
        IDENTITY,
        &mut buffer,
    );
    // Small green circle
    ellipse(
        [0.0, 1.0, 0.0, 0.7],
        [50.0, 50.0, 50.0, 50.0],
        IDENTITY,
        &mut buffer,
    );

    // Save the buffer
    buffer.save("circles.png").unwrap();
}
