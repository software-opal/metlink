use crate::data::timetable::RouteDirection;
use chrono::NaiveDate;
use std::path::{Path, PathBuf};

pub fn services_json<S: Into<String>>(data_folder: &Path) -> PathBuf {
    data_folder.join("services.json")
}
pub fn stops_json<S: Into<String>>(data_folder: &Path) -> PathBuf {
    data_folder.join("stops.json")
}

pub fn service_folder<S: Into<String>>(data_folder: &Path, service_code: S) -> PathBuf {
    data_folder.join(format!("service-{}", service_code.into()))
}
pub fn service_json<S: Into<String>>(data_folder: &Path, service_code: S) -> PathBuf {
    service_folder(data_folder, service_code).join("service.json")
}
pub fn timetables_json<S: Into<String>>(data_folder: &Path, service_code: S) -> PathBuf {
    service_folder(data_folder, service_code).join("timetables.json")
}

pub fn timetables_folder<S: Into<String>>(data_folder: &Path, service_code: S) -> PathBuf {
    service_folder(data_folder, service_code).join("timetables")
}
pub fn timetable_json<S: Into<String>>(
    data_folder: &Path,
    service_code: S,
    date: NaiveDate,
    direction: RouteDirection,
) -> PathBuf {
    timetables_folder(data_folder, service_code).join(format!(
        "{}-{}.json",
        date.format("%Y-%m-%d"),
        direction.name()
    ))
}
