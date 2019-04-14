use geo_types::{Coordinate, Point};
use std::f64::consts::{FRAC_1_PI, FRAC_PI_4, PI};

pub struct Projection {
    zoom: f64,
}

impl Projection {
    pub fn to_map_pixels(&self, point: impl Into<Point<f64>>) -> Coordinate<f64> {
        let (lon_rad, lat_rad) = point.into().to_radians().x_y();

        let map_size = 256. * FRAC_1_PI * 0.5 * f64::powf(2.0, self.zoom);

        let x = map_size * (lon_rad + PI);
        let y = map_size * (PI - f64::ln(f64::tan(FRAC_PI_4 + (lat_rad / 2.))));
        Coordinate::from((x, y))
    }
}
