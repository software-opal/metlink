use crate::{
    data::{files::service_json, Route, Stop},
    shared::ServiceMode,
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{ fs::File, path::Path};
use anyhow::{Context, Error, Result};

pub fn load_services(data_folder: &Path) -> Result<Vec<Service>> {
    let services = from_reader(File::open(data_folder.join("stops.json"))?)?;
    Ok(services)
}

pub fn load_extended_service(
    data_folder: &Path,
    service_code: String,
) -> Result<ExtendedService> {
    let service = from_reader(File::open(service_json(data_folder, service_code))?)?;
    Ok(service)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Service {
    pub code: String,
    pub last_modified: DateTime<FixedOffset>,
    pub link: String,
    pub mode: ServiceMode,
    pub name: String,
    pub schools: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExtendedService {
    pub code: String,
    pub last_modified: DateTime<FixedOffset>,
    pub mode: ServiceMode,
    pub name: String,
    pub routes: Vec<Route>,
    pub schools: Vec<String>,
    pub stops: Vec<Stop>,
}
