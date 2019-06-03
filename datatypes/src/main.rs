mod api;
mod import;
mod internal;
mod py_data;
mod route_resolver;
mod utils;

use crate::api::service::Service;
use crate::api::service_map::MapStop;
use crate::api::service_map::RouteMap;
use crate::api::service_map::ServiceMap;
use crate::import::{load_service_list, load_service_map, load_service_timetables};
use crate::internal::ServiceTimetable;
use crate::internal::ServiceTimetableEntry;
use crate::py_data::timetables::Direction;
use crate::route_resolver::stops::generate_routes_for;
use crate::utils::closest_point;
use geo::Point;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;

type BoxResult<T> = Result<T, Box<Error>>;
type StopId = String;
type StartEndRouteMap = BTreeMap<StopId, BTreeMap<StopId, Vec<usize>>>;
type StopList = Vec<StopId>;
type PointList<PT> = Vec<Point<PT>>;

fn get_timetable(service: &Service) -> BoxResult<ServiceTimetable> {
    let timetables = ServiceTimetable::from_py_timetables(load_service_timetables(service)?);
    assert_eq!(timetables.len(), 1);
    Ok(timetables.into_iter().next().unwrap())
}

fn build_start_end_map<'a>(
    stop_locations: Vec<MapStop>,
    route_maps: &Vec<RouteMap>,
) -> BoxResult<StartEndRouteMap> {
    let mut map_start_to_ends: StartEndRouteMap = BTreeMap::new();

    let stops = stop_locations
        .into_iter()
        .map(|s| (s.location.into(), s.sms))
        .collect::<Vec<(Point<_>, _)>>();

    let route_start_ends = route_maps
        .iter()
        .map(|route| {
            (
                route.path.first().unwrap().clone().into(),
                route.path.last().unwrap().clone().into(),
            )
        })
        .collect::<Vec<(Point<_>, Point<_>)>>();

    for (i, (start, end)) in route_start_ends.iter().enumerate() {
        let start = closest_point(&start, &stops).unwrap().2.to_owned();
        let end = closest_point(&end, &stops).unwrap().2.to_owned();
        map_start_to_ends
            .entry(start)
            .or_default()
            .entry(end)
            .or_default()
            .push(i);
    }
    Ok(map_start_to_ends)
}

fn process_service(
    service: &Service,
) -> Result<Vec<(Direction, StopList, PointList<f64>)>, Box<Error>> {
    let timetable = get_timetable(service)?;

    let ServiceMap {
        stop_locations,
        route_maps,
        ..
    } = load_service_map(service)?;
    let map_start_to_ends = build_start_end_map(stop_locations, &route_maps)?;
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
            (
                dir,
                route,
                generate_routes_for(&route[..], &map_start_to_ends),
            )
        })
        .collect::<Vec<_>>();

    let mut full_routes = Vec::new();
    for (dir, route, route_segments) in out {
        if route_segments.len() == 0 {
            println!("Cannot determine route segments! {:?} {:?}", dir, route);
        } else if route_segments.len() == 1 {
            let mut route_path = Vec::new();
            for i in route_segments[0].iter() {
                let mut segment_path = route_maps[*i].clone_to_point_list();
                route_path.append(&mut segment_path)
            }
            full_routes.push((dir.clone(), route.clone(), route_path));
        } else {
            // TODO
            println!("Cannot determine route segments! {:?} {:?}", dir, route);
            println!("  - {:?}", route_segments);
        }
    }

    Ok(full_routes)
}

fn main() -> Result<(), Box<Error>> {
    let services: Vec<Service> = load_service_list()?;

    for service in services {
        println!(" --- {:?}", service);
        let service =
            process_service(&service).map_err(|e| format!("{}: {:?}", service.code, e))?;
        for (dir, stops, path) in service {
            println!(
                "{:?} [{:?}, ..., {:?}] - {:?}",
                dir,
                stops[0],
                stops.last().unwrap(),
                path.len()
            );
        }
        return Ok(());
    }

    // let service_results = svcs
    //     .par_iter()
    //     .map(|svc| )
    //     .collect::<Vec<_>>();
    //
    // for r in service_results {
    //     r?;
    // }

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
