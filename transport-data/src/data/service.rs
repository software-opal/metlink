use crate::{
    data::{
        files::{service_json, services_json},
        Route,
        Stop,
    },
    shared::ServiceMode,
};
use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{fs::File, io::BufReader, path::Path};

pub fn load_services(data_folder: &Path) -> Result<Vec<Service>> {
    let services = from_reader(BufReader::new(
        File::open(services_json(data_folder))
            .with_context(|| format!("Failed to open services file"))?,
    ))
    .with_context(|| format!("Failed to read services file"))?;
    Ok(services)
}

pub fn load_extended_service(data_folder: &Path, service_code: String) -> Result<ExtendedService> {
    let service = from_reader(BufReader::new(
        File::open(service_json(data_folder, &service_code))
            .with_context(|| format!("Failed to open service file for {}", &service_code))?,
    ))
    .with_context(|| format!("Failed to read service file for {}", &service_code))?;
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
