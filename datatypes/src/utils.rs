use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::CoordinateType;
use geo::Point;
use num_traits::Float;
use crate::py_data::timetables::Direction;
use std::collections::BTreeMap;
use std::error::Error;

pub type BoxResult<T> = Result<T, Box<Error>>;
pub type StopId = String;
pub type StartEndRouteMap = BTreeMap<StopId, BTreeMap<StopId, Vec<usize>>>;
pub type StopList = Vec<StopId>;
pub type PointList<PT> = Vec<Point<PT>>;
pub type RouteInformation = (Direction, StopList, PointList<f64>);


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
    r
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
