use crate::data::files::routes_json;
use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer_pretty};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Route {
    pub end_id: String,
    pub id: String,
    pub route: Vec<RouteSegment>,
    pub start_id: String,
    pub stops: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
