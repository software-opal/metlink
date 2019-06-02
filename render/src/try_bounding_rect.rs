use geo::bounding_rect::BoundingRect;
use geo_types::Coordinate;
use geo_types::CoordinateType;
use geo_types::Rect;

pub trait TryBoundingRect<T: CoordinateType> {
    fn try_bounding_rect(&self) -> Option<Rect<T>>;
}

impl<T> TryBoundingRect<T> for BoundingRect<T, Output = Rect<T>>
where
    T: CoordinateType,
{
    fn try_bounding_rect(&self) -> Option<Rect<T>> {
        Some(self.bounding_rect())
    }
}
impl<T> TryBoundingRect<T> for BoundingRect<T, Output = Option<Rect<T>>>
where
    T: CoordinateType,
{
    fn try_bounding_rect(&self) -> Option<Rect<T>> {
        self.bounding_rect()
    }
}
impl<T> TryBoundingRect<T> for Rect<T>
where
    T: CoordinateType,
{
    fn try_bounding_rect(&self) -> Option<Rect<T>> {
        let a = self.min;
        let b = self.max;
        let (xmin, xmax) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (ymin, ymax) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        Some(Rect {
            min: Coordinate { x: xmin, y: ymin },
            max: Coordinate { x: xmax, y: ymax },
        })
    }
}
