use geo::{MultiPolygon, Polygon};
use shapefile::record::traits::MultipartShape;

pub fn nz_shapes() -> MultiPolygon<f64> {
    let reader =
        shapefile::Reader::from_path("gis\\nz-coastlines-and-islands-polygons-topo-150k.shp")
            .unwrap();
    let polys: Vec<_> = reader
        .iter_shapes_as::<shapefile::Polygon>()
        .filter_map(|polys_or_err| polys_or_err.ok())
        .flat_map(|polys| {
            polys
                .parts()
                .map(|poly| {
                    Polygon::new(
                        poly.into_iter()
                            .map(|p| (p.x, p.y))
                            .collect::<Vec<_>>()
                            .into(),
                        vec![].into(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();

    MultiPolygon::from(polys)
}
