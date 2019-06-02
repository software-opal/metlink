mod api;
mod internal;
mod py_data;
mod route_resolver;
use crate::route_resolver::stops::generate_routes_for;
use crate::api::service::Service;
use crate::api::service_map::ServiceMap;
use crate::api::stop::StopListResponse;
use crate::internal::ServiceTimetable;
use crate::internal::ServiceTimetableEntry;
use crate::py_data::timetables::Timetable;
use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::CoordinateType;
use geo::Point;
use num_traits::Float;
use rayon::prelude::*;
use serde_json::from_reader;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use std::error::Error;
use std::fs::read_dir;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

const CACHED_RESPONSES_FOLDER: &'static str = "../responses";
const SERVICE_DATA_FOLDER: &'static str = "../data";

fn service_list_data() -> io::Result<impl Read> {
    let p = Path::new(CACHED_RESPONSES_FOLDER)
        .join("https___www.metlink.org.nz_api_v1_ServiceList_.json");
    File::open(p)
}
fn stop_list_data() -> io::Result<impl Read> {
    let p =
        Path::new(CACHED_RESPONSES_FOLDER).join("https___www.metlink.org.nz_api_v1_StopList_.json");
    File::open(p)
}
fn service_map_data(code: &str) -> io::Result<impl Read> {
    let p = Path::new(CACHED_RESPONSES_FOLDER).join(format!(
        "https___www.metlink.org.nz_api_v1_ServiceMap_{}.json",
        code.to_uppercase()
    ));
    File::open(p)
}

fn load_service_map(svc: &Service) -> Result<ServiceMap, Box<Error>> {
    Ok(from_reader(service_map_data(&svc.code)?)?)
}

fn load_service_timetable(timetable_json: &Path) -> Result<Option<Timetable>, Box<Error>> {
    if timetable_json.is_file() {
        Ok(Some(from_reader(File::open(timetable_json)?)?))
    } else {
        Ok(None)
    }
}

fn load_service_timetables(svc: &Service) -> Result<Vec<Timetable>, Box<Error>> {
    let timetable_folder = Path::new(SERVICE_DATA_FOLDER)
        .join(format!("service-{}/timetables/", svc.code.to_uppercase()));
    if timetable_folder.is_dir() {
        let dir_items = read_dir(timetable_folder)?
            .into_iter()
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

fn closest_point<'a, PT, S>(
    target: &Point<PT>,
    points: &'a [(Point<PT>, S)],
) -> Option<((&'a Point<PT>, PT, &'a S))>
where
    PT: Float + CoordinateType,
    S: Sized,
{
    let mut r = Option::None;
    for (p, v) in points {
        let dist = target.euclidean_distance(p);
        r = match r {
            None => Some((p, dist, v)),
            Some((op, odist, ov)) => {
                if odist < dist {
                    Some((p, dist, v))
                } else {
                    Some((op, odist, ov))
                }
            }
        };
    }

    return r;
}

fn process_service(svc: &Service) -> Result<(), Box<Error>> {
    let timetables = ServiceTimetable::from_py_timetables(load_service_timetables(svc)?);
    assert_eq!(timetables.len(), 1);
    let timetable = timetables.into_iter().next().unwrap();
    let mut map_start_to_ends: BTreeMap<&str, BTreeMap<&str, Vec<usize>>> = BTreeMap::new();
    let ServiceMap {
        stop_locations,
        route_maps,
        ..
    } = load_service_map(svc)?;
    let stops = stop_locations
        .into_iter()
        .map(|s| (s.location.into(), s.sms))
        .collect::<Vec<(Point<_>, _)>>();
    for (i, route) in route_maps.iter().enumerate() {
        let start = closest_point(&route.path.first().unwrap().clone().into(), &stops)
            .unwrap()
            .2;
        let end = closest_point(&route.path.last().unwrap().clone().into(), &stops)
            .unwrap()
            .2;
        map_start_to_ends
            .entry(start)
            .or_default()
            .entry(end)
            .or_default()
            .push(i);
    }
    println!("{:?}", map_start_to_ends);
    let routes = timetable
        .entries
        .into_iter()
        .map(
            |ServiceTimetableEntry {
                 direction,
                 stops,
                 times: _,
             }| (direction, stops),
        )
        .collect::<BTreeSet<_>>();

    println!("Routes: {:?}", routes);

    let out = routes
        .par_iter()
        .map(|(dir, route)| {
            (dir, route, generate_routes_for(&route[..], &map_start_to_ends))
        })
        .collect::<Vec<_>>();


    for (dir, route, route_segments) in out {
        println!("{:?} {:?} -> {:?}", dir, route, route_segments);
    }

    Ok(())
}

fn main() -> Result<(), Box<Error>> {
    let svcs: Vec<Service> = from_reader(service_list_data()?)?;
    let stops: StopListResponse = from_reader(stop_list_data()?)?;
    println!("Stops: {:?}", stops.stops.len());
    println!("Services: {:?}", svcs.len());

    let service_results = svcs
        .par_iter()
        .map(|svc| process_service(&svc).map_err(|e| format!("{}: {:?}", svc.code, e)))
        .collect::<Vec<_>>();

    for r in service_results {
        r?;
    }

    // let services = svcs
    //     .into_iter()
    //     .map(|s| (s.code.clone(), s))
    //     .collect::<BTreeMap<_, _>>();
    // let maps = maps_?
    //     .into_iter()
    //     .map(|m| (m.code.clone(), m))
    //     .collect::<BTreeMap<_, _>>();
    // let timetables = timetables_?
    //     .into_iter()
    //     .map(|t| (t.code.clone(), t))
    //     .collect::<BTreeMap<_, _>>();
    //
    // let timetabled_routes = timetables
    //     .clone()
    //     .into_iter()
    //     .map(|(c, ts)| {
    //         (
    //             c,
    //             ts.entries
    //         )
    //     })
    //     .collect::<BTreeMap<_, _>>();
    //
    // for map in maps {
    //     // println!(
    //     //     "Map for {:?} contains {} paths",
    //     //     map.code,
    //     //     map.route_maps.len()
    //     // );
    // }

    Ok(())
}
