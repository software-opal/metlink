mod routing;
mod service;

use crate::routing::find_route_components_used_in_stop_list;
use crate::service::Service;
use crate::service::Timetable;
use crate::service::TimetabledService;
use serde_json::from_reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{read_dir, File};
use std::path::PathBuf;

const DATA: &'static str = "../data";

fn load_data(folder: &PathBuf) -> Result<Option<(Service, Vec<TimetabledService>)>, Box<Error>> {
    let service_json = folder.join("service.json");
    let timetable_folder = folder.join("timetables");

    if service_json.is_file() {
        let svc: Service = from_reader(File::open(service_json)?)?;
        let mut ttbl_svcs = vec![];
        Ok(Some((svc, ttbl_svcs)))
    } else {
        Ok(None)
    }
}

fn main() -> Result<(), Box<Error>> {
    for e in read_dir(DATA)? {
        let service_folder = e?.path();
        println!("{:?}", service_folder);
        let (service, timetables) = match load_data(&service_folder)? {
            Some(a) => a,
            None => continue,
        };
        service.regenerate_routes();
        println!("Loaded data");
        let components = service
            .routes
            .iter()
            .map(|r| &r.stops[..])
            .collect::<Vec<&[String]>>();
        let mut timetabled_routes = HashMap::new();

        for ttbl_svc in timetables {
            match timetabled_routes.get(&ttbl_svc.stops) {
                Some(_) => {}
                None => {
                    println!("Stops: {:?}", ttbl_svc.stops);
                    let route_components =
                        find_route_components_used_in_stop_list(&ttbl_svc.stops, &components);
                    println!("Found: {:?}", route_components);
                    if route_components == None {
                        println!("Failed to find components!");
                        return Ok(());
                    }
                    timetabled_routes.insert(ttbl_svc.stops, route_components);
                }
            };
        }
    }
    Ok(())
}
