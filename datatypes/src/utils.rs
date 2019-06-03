use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::CoordinateType;
use geo::Point;
use num_traits::Float;

pub fn closest_point<'a, PT, S>(
    target: &Point<PT>,
    points: &'a [(Point<PT>, S)],
) -> Option<(&'a Point<PT>, PT, &'a S)>
where
    PT: Float + CoordinateType,
    S: Sized,
{
    let mut r = Option::None;
    for (p, v) in points {
        let dist = target.euclidean_distance(p);
        r = match r {
            None => Some((p, dist, v)),
            Some((op, odist, ov)) => {
                if odist > dist {
                    Some((p, dist, v))
                } else {
                    Some((op, odist, ov))
                }
            }
        };
    }
    return r;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closest_point() {
        let target = &(1.0f64, 1.0f64).into();
        let points = [
            ((2.0f64, 2.0f64).into(), "a"),
            ((3.0f64, 1.0f64).into(), "b"),
            ((5.0f64, 5.0f64).into(), "c"),
        ];
        assert_eq!(closest_point(target, &points[0..0]), None);

        let (point, _dist, id) = closest_point(target, &points).unwrap();
        assert_eq!((*point, *id), points[0]);

        let (point, _dist, id) = closest_point(target, &points[1..]).unwrap();
        assert_eq!((*point, *id), points[1]);
    }
}
