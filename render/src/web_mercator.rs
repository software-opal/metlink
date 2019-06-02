use geo_types::{Coordinate, Point};
use std::f64::consts::{FRAC_1_PI, FRAC_PI_4, PI};

#[derive(PartialEq, Clone, Debug)]
pub struct Projection {
    pub zoom: f64,
}

impl Projection {
    pub fn to_map_pixels(&self, point: impl Into<Point<f64>>) -> Coordinate<f64> {
        let (lon_rad, lat_rad) = point.into().to_radians().x_y();

        let map_size = 256. * FRAC_1_PI * 0.5 * f64::powf(2.0, self.zoom);

        let x = map_size * (lon_rad + PI);
        let y = map_size * (PI - f64::ln(f64::tan(FRAC_PI_4 + (lat_rad / 2.))));
        Coordinate::from((x, y))
    }

    pub fn coord_mapper(&self, point: &(f64, f64)) -> (f64, f64) {
        self.to_map_pixels((point.0, point.1)).x_y()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_projection(
        zoom: f64,
        lon_lat: impl Into<Point<f64>>,
        x_y: impl Into<Coordinate<f64>>,
        epsilon: f64,
    ) {
        let (x, y) = Projection { zoom: zoom }.to_map_pixels(lon_lat).x_y();
        let (target_x, target_y) = x_y.into().x_y();
        let abs_difference_x = (x - target_x).abs();
        let abs_difference_y = (y - target_y).abs();

        assert!(abs_difference_x <= epsilon, "X: {} != {}", x, target_x);
        assert!(abs_difference_y <= epsilon, "Y: {} != {}", y, target_y);
    }

    #[test]
    fn simple_test() {
        test_projection(
            19.,
            (-1.29776269197464, 50.699935334309245),
            (260254. * 256., 176212. * 256.),
            1.0, // Allow 1px out; because I'm doing this by eye.
        );
        test_projection(
            13.,
            (174.77033615112305, -41.27780646738182),
            (8073. * 256., 5129. * 256.),
            1.0, // Allow 1px out; because I'm doing this by eye.
        );
        test_projection(
            19.,
            (174.77943420410156, -41.28554642309004),
            (516685. * 256., 328271. * 256.),
            1.0, // Allow 1px out; because I'm doing this by eye.
        );
    }
}
