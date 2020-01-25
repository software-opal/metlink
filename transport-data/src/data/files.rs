use std::path::{Path, PathBuf};

pub fn service_folder<P: Into<&Path>, S: Into<String>>(
    data_folder: &P,
    service_code: S,
) -> PathBuf {
    data_folder
        .into()
        .join(format!("service-{}", service_code.into()))
}
pub fn timetables_folder<P: Into<&Path>, S: Into<String>>(
    data_folder: &P,
    service_code: S,
) -> std::path::PathBuf {
    service_folder(data_folder, service_code).join("timetables")
}
pub fn service_json<P: Into<&Path>, S: Into<String>>(
    data_folder: &P,
    service_code: S,
) -> std::path::PathBuf {
    service_folder(data_folder, service_code).join("service.json")
}
