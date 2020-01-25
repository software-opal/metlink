use crate::{
    data::{files::service_json, Route, Stop},
    shared::ServiceMode,
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{error::Error, fs::File, path::Path};

pub fn load_services(data_folder: &Path) -> Result<Vec<Service>, Box<dyn Error>> {
    let services = from_reader(File::open(data_folder.join("stops.json"))?)?;
    Ok(services)
}

pub fn load_extended_service(
    data_folder: &Path,
    service_code: String,
) -> Result<ExtendedService, Box<dyn Error>> {
    let service = from_reader(File::open(service_json(data_folder, service_code))?)?;
    Ok(service)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Service {
    code: String,
    last_modified: DateTime<FixedOffset>,
    link: String,
    mode: ServiceMode,
    name: String,
    schools: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExtendedService {
    code: String,
    last_modified: DateTime<FixedOffset>,
    mode: ServiceMode,
    name: String,
    routes: Vec<Route>,
    schools: Vec<String>,
    stops: Vec<Stop>,
}
