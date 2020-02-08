use crate::data_utils::*;
use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, ToPrimitive};
use metlink_transport_data::data::{Route, RouteSegment, Stop};
use std::{collections::HashSet, ops::Add};

fn distance(a: &Stop, segment: &RouteSegment) -> f64 {
    ((segment.lat.to_f64().unwrap() - a.lat.to_f64().unwrap()).powi(2)
        + (segment.lon.to_f64().unwrap() - a.lon.to_f64().unwrap()).powi(2))
    .sqrt()
}

fn order_route_idxs_by_closest(location: &Stop, route: &Route) -> Vec<(usize, f64)> {
    let mut route_idxs = route
        .route
        .iter()
        .map(|point| distance(location, point))
        .enumerate()
        .collect::<Vec<_>>();
    route_idxs.sort_unstable_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    route_idxs
}

fn find_bestest_route_stops(
    min_idx: usize,
    ordered_stop_idxs: &[Vec<(usize, f64)>],
) -> Option<(f64, Vec<usize>)> {
    if ordered_stop_idxs.is_empty() {
        return Some((0.0, Vec::new()));
    }
    let this_idxs = &ordered_stop_idxs[0];
    if min_idx > this_idxs.len() {
        return None;
    }
    let other_idxs = &ordered_stop_idxs[1..];
    for (idx, dist) in this_idxs {
        if *idx >= min_idx {
            if let Some((sub_dist, mut stop_idxs)) = find_bestest_route_stops(idx + 1, other_idxs) {
                stop_idxs.push(*idx);
                return Some((dist + sub_dist, stop_idxs));
            }
        }
    }
    None
}

fn find_route_stops(
    route_segment: &[StopId],
    stops: &StopList,
    route: &Route,
) -> Option<(f64, Vec<usize>)> {
    let ordered_routes = route_segment
        .iter()
        .map(|id| stops.get(id).unwrap())
        .map(|stop| order_route_idxs_by_closest(&stop, &route))
        .collect::<Vec<_>>();
    match find_bestest_route_stops(0, &ordered_routes) {
        Some((dist, mut stop_indexes)) => {
            stop_indexes.reverse();
            Some((dist, stop_indexes))
        }
        None => None,
    }
}

pub(crate) fn find_next_checks<'a>(
    indent: String,
    timetabled_route: &[StopId],
    possible_ends: &'a RoutesByEnd,
    stops: &StopList,
) -> Result<Vec<(usize, (&'a StopId, usize), f64, std::vec::Vec<usize>)>> {
    let mut new_to_check = Vec::new();
    for (end, routes) in possible_ends {
        let end_pos = match timetabled_route[1..].iter().position(|v| v == end) {
            None => continue,
            Some(pos) => pos,
        } + 1;
        #[cfg(test)]
        println!(
            "{}Finding route for {:?}; Checking end: {:?}, found at {}",
            indent, timetabled_route, end, end_pos,
        );
        let route_segment = &timetabled_route[1..end_pos];
        for (idx, route) in routes.iter().enumerate() {
                match find_route_stops(route_segment, stops, route) {
                    Some((closest_approach, stop_indexes)) => {
                    #[cfg(test)]
                        println!(
                            "{}Found a route: {:?}",
                            indent, (closest_approach, &stop_indexes)
                        );

                        new_to_check.push((end_pos, (end, idx), closest_approach, stop_indexes))
                    }
                    None => {
                    #[cfg(test)]
                        println!(
                            "{}Could not find route for segment [{}..{}] = {:?}",
                            indent, 1, end_pos, route_segment
                        );
                    }
                }
        }
    }
    if new_to_check.is_empty() {
        anyhow::bail!("Failed to find any routes.");
    }
    Ok(new_to_check)
}

fn do_find_route(
    indent: String,
    timetabled_route: &[StopId],
    possible_routes: &RoutesByStartEnd,
    stops: &StopList,
) -> Result<(f64, Vec<Route>, Vec<Vec<usize>>)> {
    if timetabled_route.len() <= 1 {
        return Ok((0.0, Vec::new(), Vec::new()));
    }
    let start_id = &timetabled_route[0];
    let start_stop = match stops.get(start_id) {
        None => anyhow::bail!(
            "Cannot find start_id({:?}) in stops: {:?}",
            start_id,
            stops.keys().collect::<Vec<_>>()
        ),
        Some(stop) => stop,
    };
    let possible_ends = match possible_routes.get(start_id) {
        None => anyhow::bail!(
            "Cannot find start_id({:?}) in possible routes: {:?}",
            start_id,
            possible_routes.keys().collect::<Vec<_>>()
        ),
        Some(map) => map,
    };
    #[cfg(test)]
    println!(
        "{}Finding route for {:?}; possible ends: {:?}",
        indent,
        timetabled_route,
        possible_ends.keys().collect::<Vec<_>>()
    );
    let new_to_check = find_next_checks(
        (" ".to_owned() + &indent).to_string(),
        timetabled_route,
        possible_ends,
        stops,
    )?;
    #[cfg(test)]
    println!("{}Found suitable routes: {:?}", indent, new_to_check);
    let mut best_closest = std::f64::INFINITY;
    let mut best_route = None;

    for (new_start, (end_stop_id, route_idx), closest_app, stop_indexes) in new_to_check {
        let route = &possible_ends.get(end_stop_id).unwrap()[route_idx];
        let end_stop = match stops.get(end_stop_id) {
            None => anyhow::bail!(
                "Cannot find start_id({:?}) in stops: {:?}",
                end_stop_id,
                stops.keys().collect::<Vec<_>>()
            ),
            Some(stop) => stop,
        };
        #[cfg(test)]
        println!(
            "{}Checking route: {:?}",
            indent,
            (new_start, end_stop_id, route_idx)
        );
        let (sub_closest, mut sub_best_route, mut sub_stop_idxs) = match do_find_route(
            ("  ".to_owned() + &indent).to_string(),
            &timetabled_route[new_start..],
            possible_routes,
            stops,
        ) {
            Err(e) => {
                #[cfg(test)]
                println!(
                    "{}Failed to find route starting at {}: {}",
                    indent, new_start, e
                );
                continue;
            }
            Ok(value) => value,
        };
        let new_closest = distance(start_stop, &route.route[0])
            + closest_app
            + distance(end_stop, &route.route.last().unwrap())
            + sub_closest;
        #[cfg(test)]
        println!(
            "{}Closest approach for {:?} was: {}",
            indent,
            (new_start, end_stop_id, route_idx),
            new_closest
        );
        if new_closest < best_closest {
            best_closest = new_closest;
            sub_best_route.push(route.clone());
            sub_stop_idxs.push(stop_indexes);
            best_route.replace((sub_best_route, sub_stop_idxs));
        }
    }
    match best_route {
        None => anyhow::bail!("Failed to find suitable route"),
        Some((route_list, stop_idxs)) => Ok((best_closest, route_list, stop_idxs)),
    }
}

pub fn find_route(
    timetabled_route: &[StopId],
    possible_routes: &RoutesByStartEnd,
    stops: &StopList,
) -> Result<(Vec<Route>, Route)> {
    let (_, mut route_list, mut stop_idxs) = do_find_route(
        "".to_string(),
        timetabled_route,
        &possible_routes,
        stops,
    )
    .with_context(|| {
        format!(
            "Cannot determine best route for timetabled_route: {:?}",
            timetabled_route
        )
    })?;
    let end_id = timetabled_route[timetabled_route.len() - 1].to_string();
    let start_id = timetabled_route[0].to_string();
    anyhow::ensure!(
        route_list.len() == stop_idxs.len(),
        "Route list does not have the same number of elements as stop_idxs"
    );
    route_list.reverse();
    stop_idxs.reverse();
    let mut current_stop_index = 0;

    let mut flattened_route =
        Vec::with_capacity(route_list.iter().map(|r| r.route.len()).fold(0, usize::add));
    for (stop_idx_segment, route_segment) in stop_idxs.into_iter().zip(route_list.iter()) {
        // assert!(stop_idx_segment.iter().is_sorted());
        if flattened_route.is_empty() {
            flattened_route.push(route_segment.route[0].clone());
            current_stop_index += 1;
        }
        let mut prev = 1;
        for &idx in &stop_idx_segment {
            if idx < prev {
                anyhow::bail!(
                    "Cannot segment route: {:?}, {:?}",
                    timetabled_route,
                    stop_idx_segment
                );
            }
            flattened_route.extend_from_slice(&route_segment.route[prev..idx]);
            flattened_route.push(RouteSegment {
                stop: Some(timetabled_route[current_stop_index].clone()),
                ..route_segment.route[idx].clone()
            });
            current_stop_index += 1;
            prev = idx + 1;
        }
        flattened_route.extend_from_slice(&route_segment.route[prev..]);
        current_stop_index += 1;
    }
    let route_id = route_list
        .iter()
        .map(|r| r.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    Ok((
        route_list,
        Route {
            end_id,
            id: route_id,
            route: flattened_route,
            start_id,
            stops: timetabled_route.to_vec(),
        },
    ))
}

#[cfg(test)]
mod tests;
