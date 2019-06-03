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
use crate::route_resolver::best_route::find_best_route;
use crate::route_resolver::stops::generate_routes_for;
use geo::Point;
use crate::utils::{closest_point, BoxResult, RouteInformation, StartEndRouteMap, StopId};
use std::collections::{BTreeMap, BTreeSet};
// use rayon::prelude::*;

fn get_timetable(service: &Service) -> BoxResult<ServiceTimetable> {
    let timetables = ServiceTimetable::from_py_timetables(load_service_timetables(service)?);
    assert_eq!(timetables.len(), 1);
    Ok(timetables.into_iter().next().unwrap())
}

fn build_start_end_map(
    stop_locations: &BTreeMap<StopId, Point<f64>>,
    route_maps: &[RouteMap],
) -> BoxResult<StartEndRouteMap> {
    let mut map_start_to_ends: StartEndRouteMap = BTreeMap::new();

    let stops = stop_locations
        .into_iter()
        .map(|(sms, loc)| (*loc, sms.clone()))
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


fn process_service(service: &Service) -> BoxResult<Vec<RouteInformation>> {
    let timetable = get_timetable(service)?;

    let ServiceMap {
        stop_locations,
        route_maps,
        ..
    } = load_service_map(service)?;


    let stop_id_to_loc = stop_locations.into_iter().map(|s| (s.sms, s.location.into())).collect::<BTreeMap<StopId, Point<_>>>();

    let map_start_to_ends = build_start_end_map(&stop_id_to_loc, &route_maps)?;
    let routes = timetable
        .entries
        .into_iter()
        .map(
            |ServiceTimetableEntry {
                 direction, stops, ..
             }| (direction, stops),
        )
        .collect::<BTreeSet<_>>();

    let out = routes
        .iter()
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
        if route_segments.is_empty() {
            println!(
                "Cannot determine route segments for {:?}! {:?} {:?} -- {}",
                service.code,
                dir,
                route,
                route_segments.len()
            );
            println!("{:?}", map_start_to_ends);

        } else {
            let best_route_idx =
                find_best_route(route, &route_segments, &map_start_to_ends, &stop_id_to_loc, &route_maps);
            match best_route_idx {
                None => {
                    println!(
                        "Cannot determine route segments for {:?}! {:?} {:?}",
                        service.code, dir, route
                    );
                    println!("  - {:?}", route_segments);
                    println!("{:?}", map_start_to_ends);
                }
                Some(idx) => {
                    let mut route_path = Vec::new();
                    for i in route_segments[idx].iter() {
                        let mut segment_path = route_maps[*i].clone_to_point_list();
                        route_path.append(&mut segment_path)
                    }
                    full_routes.push((dir.clone(), route.clone(), route_path));
                }
            }
        }
    }

    Ok(full_routes)
}

fn main() -> BoxResult<()> {
    let services: Vec<Service> = load_service_list()?;

    let service_results = {
        let results = services
            // .par_iter()
            .iter()
            .map(|svc| process_service(&svc).map_err(|e| format!("{}: {:?}", svc.code, e)))
            .collect::<Vec<_>>();
        let mut sr = Vec::with_capacity(services.len());
        for (service, result) in services.iter().zip(results) {
            sr.push((service, result?));
        }
        sr
    };
    for (service, results) in service_results {
        println!(" --- {:?}", service);
        for (dir, stops, path) in results {
            println!(
                "{:?} [{:?}, ..., {:?}] - {:?}",
                dir,
                stops[0],
                stops.last().unwrap(),
                path.len()
            );
        }
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
