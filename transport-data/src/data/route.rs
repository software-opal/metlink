use crate::data::files::routes_json;
use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer_pretty};
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RouteSegment {
    pub lat: BigDecimal,
    pub lon: BigDecimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
}

pub fn load_routes<S: Into<String>>(data_folder: &Path, service_code: S) -> Result<Vec<Route>> {
    let service_code = service_code.into();
    let timetables = from_reader(BufReader::new(
        File::open(routes_json(data_folder, &service_code))
            .with_context(|| format!("Failed to open service file for {}", &service_code))?,
    ))
    .with_context(|| format!("Failed to read service file for {}", &service_code))?;
    Ok(timetables)
}

pub fn save_routes<S: Into<String>>(
    data_folder: &Path,
    service_code: S,
    routes: &[Route],
) -> Result<()> {
    let service_code = service_code.into();
    let timetables = to_writer_pretty(
        BufWriter::new(
            File::create(routes_json(data_folder, &service_code))
                .with_context(|| format!("Failed to open service file for {}", &service_code))?,
        ),
        routes,
    )
    .with_context(|| format!("Failed to read service file for {}", &service_code))?;
    Ok(timetables)
}
