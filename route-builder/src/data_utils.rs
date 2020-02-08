use anyhow::Result;
use chrono::{DateTime, FixedOffset, NaiveDate};
use metlink_transport_data::data::{
    load_extended_service,
    load_timetables,
    ExtendedService,
    Route,
    RouteDirection,
    Service,
    Stop,
    Timetable,
};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};
pub type StopId = String;
pub type StopList = BTreeMap<StopId, Stop>;
pub fn load_stops(data_folder: &Path) -> Result<StopList> {
    let stops = metlink_transport_data::data::load_stops(data_folder)?;
    Ok(stops
        .into_iter()
        .map(|stop| (stop.sms.clone(), stop))
        .collect())
}

pub type ServiceId = String;
pub type ServiceList = BTreeMap<ServiceId, Service>;
pub fn load_services(data_folder: &Path) -> Result<ServiceList> {
    let services = metlink_transport_data::data::load_services(data_folder)?;
    Ok(services.into_iter().map(|s| (s.code.clone(), s)).collect())
}

pub fn load_ext_service(data_folder: &Path, service_code: &ServiceId) -> Result<ExtendedService> {
    load_extended_service(data_folder, service_code.to_string())
}

pub fn load_timetable(data_folder: &Path, service_code: &ServiceId) -> Result<Vec<Timetable>> {
    load_timetables(data_folder, service_code)
}

pub type TimetableTiming = (NaiveDate, RouteDirection, Vec<DateTime<FixedOffset>>);
pub type TimetablesByRoute = HashMap<Vec<String>, Vec<TimetableTiming>>;
pub fn organise_timetables(ttbls: Vec<Timetable>, after: Option<NaiveDate>) -> TimetablesByRoute {
    let mut timetables = HashMap::with_capacity(4);

    for timetable in ttbls.into_iter() {
        let date = timetable.day;
        match after {
            None => {}
            Some(after) => {
                if date < after {
                    continue;
                }
            }
        }
        let direction = timetable.direction;
        for route in timetable.timetables {
            timetables
                .entry(route.stops)
                .or_insert_with(|| Vec::with_capacity(10))
                .push((date, direction, route.times));
        }
    }
    for (_, routes) in timetables.iter_mut() {
        routes.shrink_to_fit()
    }
    timetables
}

pub type RoutesByEnd = BTreeMap<StopId, Vec<Route>>;
pub type RoutesByStartEnd = BTreeMap<StopId, RoutesByEnd>;
pub fn organise_routes(service: ExtendedService) -> RoutesByStartEnd {
    let mut routes: RoutesByStartEnd = BTreeMap::new();
    for route in service.routes {
        routes
            .entry(route.start_id.clone())
            .or_default()
            .entry(route.end_id.clone())
            .or_default()
            .push(route)
    }
    routes
}
