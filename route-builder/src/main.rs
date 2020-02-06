use anyhow::{Context, Result};
use metlink_transport_data::data::{
    load_extended_service,
    load_services,
    load_stops,
    load_timetables,
    ExtendedService,
    Stop,
    Timetable,
};
use rayon::prelude::*;
use std::{collections::BTreeMap, path::Path};

fn collect_results<I: IntoIterator<Item = Result<T>>, T>(iter: I) -> Result<Vec<T>> {
    let iter = iter.into_iter();
    let mut result = Vec::with_capacity(iter.size_hint().0);
    for item in iter {
        result.push(item?);
    }
    return Ok(result);
}

#[derive(Debug, Clone, PartialEq)]
struct StopList {
    stops_by_sms: BTreeMap<String, Stop>,
}

impl StopList {
    pub fn load_stops(data_folder: &Path) -> Result<Self> {
        let stops = load_stops(data_folder)?;
        Ok(StopList {
            stops_by_sms: stops
                .into_iter()
                .map(|stop| (stop.sms.clone(), stop))
                .collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ServiceList {
    services_by_sms: BTreeMap<String, ExtendedService>,
}

impl ServiceList {
    pub fn load_services(data_folder: &Path) -> Result<Self> {
        let services = load_services(data_folder)?;

        let ext_services =
                collect_results(
                    services
                        .par_iter()
                        .map(|service| {
                            load_extended_service(data_folder, service.code.clone())
                                .with_context(|| {
                                    format!("Failed to load service {}", &service.code)
                                })
                                .map(|v| (service.code.clone(), v))
                        })
                        .collect::<Vec<_>>(),
                )
        ;

        Ok(ServiceList {
            services_by_sms: ext_services?.into_iter().collect(),
        })
    }
}





fn main() -> Result<()> {
    let before = std::time::Instant::now();
    let folder = Path::new("./data/");
    let (stops, services) = rayon::join(
        || StopList::load_stops(folder),
        || ServiceList::load_services(folder),
    );
    let (stops, services) = (stops?, services?);
    println!("{:?}", std::time::Instant::now().duration_since( before));
    Ok(())
}
