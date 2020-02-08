use crate::data::files::{routes_geojson, routes_json};
use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, ToPrimitive};
use geojson::{feature, Feature, FeatureCollection, Geometry, Position, Value};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer_pretty, Map};
use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub start_id: String,
    pub end_id: String,
    pub id: String,
    pub route: Vec<RouteSegment>,
    pub stops: Vec<String>,
}
impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Route {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else {
            self.start_id
                .cmp(&other.start_id)
                .then_with(|| self.end_id.cmp(&other.end_id))
                .then_with(|| self.id.cmp(&other.id))
        }
    }
}
impl Into<Feature> for &Route {
    fn into(self) -> Feature {
        let mut props = Map::new();
        props.insert("start".to_string(), self.start_id.clone().into());
        props.insert("end".to_string(), self.end_id.clone().into());
        props.insert("stops".to_string(), self.stops.clone().into());
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::LineString(
                self.route
                    .iter()
                    .map(|rs| rs.into())
                    .collect::<Vec<Position>>(),
            ))),
            id: Some(feature::Id::String(self.id.clone())),
            properties: Some(props),
            foreign_members: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RouteSegment {
    pub lat: BigDecimal,
    pub lon: BigDecimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
}

impl Into<Position> for &RouteSegment {
    fn into(self) -> Position {
        vec![self.lon.to_f64().unwrap(), self.lat.to_f64().unwrap()]
    }
}

pub fn load_routes<S: Into<String>>(data_folder: &Path, service_code: S) -> Result<Vec<Route>> {
    let service_code = service_code.into();
    let routes = from_reader(BufReader::new(
        File::open(routes_json(data_folder, &service_code))
            .with_context(|| format!("Failed to open service file for {}", &service_code))?,
    ))
    .with_context(|| format!("Failed to read service file for {}", &service_code))?;
    Ok(routes)
}

pub fn save_routes<S: Into<String>>(
    data_folder: &Path,
    service_code: S,
    routes: &[Route],
) -> Result<()> {
    let service_code = service_code.into();
    to_writer_pretty(
        BufWriter::new(
            File::create(routes_json(data_folder, &service_code))
                .with_context(|| format!("Failed to open service file for {}", &service_code))?,
        ),
        routes,
    )
    .with_context(|| format!("Failed to read service file for {}", &service_code))?;
    Ok(())
}

pub fn save_routes_geojson<S: Into<String>>(
    data_folder: &Path,
    service_code: S,
    routes: &[Route],
    mut extra_features: Vec<Feature>,
) -> Result<()> {
    let service_code = service_code.into();
    let mut route_features = routes.iter().map(|r| r.into()).collect::<Vec<_>>();
    route_features.append(&mut extra_features);
    to_writer_pretty(
        BufWriter::new(
            File::create(routes_geojson(data_folder, &service_code))
                .with_context(|| format!("Failed to open service file for {}", &service_code))?,
        ),
        &FeatureCollection {
            bbox: None,
            features: route_features,
            foreign_members: None,
        },
    )
    .with_context(|| format!("Failed to read service file for {}", &service_code))?;
    Ok(())
}
