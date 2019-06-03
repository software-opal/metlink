use crate::api::service::Service;
use crate::api::service_map::ServiceMap;

use crate::py_data::timetables::Timetable;
use crate::utils::BoxResult;
use serde_json::from_reader;

use rayon::prelude::*;
use std::fs::File;
use std::fs::read_dir;
use std::io;
use std::io::Read;

use std::path::{Path, PathBuf};

fn responses_folder() -> PathBuf {
    let p = PathBuf::from("../responses");
    if p.exists() {
        p
    } else {
        PathBuf::from("./responses")
    }
}
fn data_folder() -> PathBuf {
    let p = PathBuf::from("../data");
    if p.exists() {
        p
    } else {
        PathBuf::from("./data")
    }
}

fn service_list_data() -> io::Result<impl Read> {
    let p = responses_folder().join("https___www.metlink.org.nz_api_v1_ServiceList_.json");
    File::open(p)
}
fn service_map_data(code: &str) -> io::Result<impl Read> {
    let p = responses_folder().join(format!(
        "https___www.metlink.org.nz_api_v1_ServiceMap_{}.json",
        code.to_uppercase()
    ));
    File::open(p)
}

pub fn load_service_list() -> BoxResult<Vec<Service>> {
    Ok(from_reader(service_list_data()?)?)
}

pub fn load_service_map(svc: &Service) -> BoxResult<ServiceMap> {
    Ok(from_reader(service_map_data(&svc.code)?)?)
}

pub fn load_service_timetable(timetable_json: &Path) -> BoxResult<Option<Timetable>> {
    if timetable_json.is_file() {
        Ok(Some(from_reader(File::open(timetable_json)?)?))
    } else {
        Ok(None)
    }
}

pub fn load_service_timetables(svc: &Service) -> BoxResult<Vec<Timetable>> {
    let timetable_folder =
        data_folder().join(format!("service-{}/timetables/", svc.code.to_uppercase()));
    if timetable_folder.is_dir() {
        let dir_items = read_dir(timetable_folder)?
            .filter_map(|f| f.ok().map(|f| f.path().to_path_buf()))
            .collect::<Vec<_>>();

        Ok(dir_items
            .par_iter()
            .map(|f| load_service_timetable(f).map_err(|e| format!("{}: {:?}", svc.code, e)))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<Option<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>())
    } else {
        Ok(vec![])
    }
}
