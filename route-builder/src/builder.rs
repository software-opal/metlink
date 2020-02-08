use crate::data_utils::*;
use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, ToPrimitive};
use metlink_transport_data::data::{Route, RouteSegment};
use std::{collections::HashSet, ops::Add};

fn order_route_idxs_by_closest(
    location: (&BigDecimal, &BigDecimal),
    route: &Route,
) -> Vec<(usize, f64)> {
    let (lat, lon) = (location.0.to_f64().unwrap(), location.1.to_f64().unwrap());
    let mut route_idxs = route
        .route
        .iter()
        .map(|point| {
            ((point.lat.to_f64().unwrap() - lat).powi(2)
                + (point.lon.to_f64().unwrap() - lon).powi(2))
            .sqrt()
        })
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
        .map(|stop| order_route_idxs_by_closest((&stop.lat, &stop.lon), &route))
        .collect::<Vec<_>>();
    match find_bestest_route_stops(0, &ordered_routes) {
        Some((dist, mut stop_indexes)) => {
            stop_indexes.reverse();
            Some((dist, stop_indexes))
        }
        None => None,
    }
}

fn find_next_checks<'a>(
    timetabled_route: &[StopId],
    possible_ends: &'a RoutesByEnd,
    stops: &StopList,
    visited_route_ids: &HashSet<String>,
) -> Result<Vec<(usize, (&'a StopId, usize), f64, std::vec::Vec<usize>)>> {
    let mut new_to_check = Vec::new();
    for (end, routes) in possible_ends {
        let end_pos = match timetabled_route[1..].iter().position(|v| v == end) {
            None => continue,
            Some(pos) => pos,
        } + 1;
        let route_segment = &timetabled_route[1..end_pos];
        for (idx, route) in routes.iter().enumerate() {
            if !visited_route_ids.contains(&route.id) {
                match find_route_stops(route_segment, stops, route) {
                    Some((closest_approach, stop_indexes)) => {
                        new_to_check.push((end_pos, (end, idx), closest_approach, stop_indexes))
                    }
                    None => {}
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
    visited_route_ids: HashSet<String>,
) -> Result<(f64, Vec<Route>, Vec<Vec<usize>>)> {
    if timetabled_route.len() <= 1 {
        return Ok((0.0, Vec::new(), Vec::new()));
    }
    let start_id = &timetabled_route[0];
    let possible_ends = match possible_routes.get(start_id) {
        None => anyhow::bail!(
            "Cannot find start_id({:?}) in possible routes: {:?}",
            start_id,
            possible_routes.keys().collect::<Vec<_>>()
        ),
        Some(map) => map,
    };
    println!(
        "{}Finding route for {:?}; possible ends: {:?}",
        indent,
        timetabled_route,
        possible_ends.keys().collect::<Vec<_>>()
    );
    let new_to_check = find_next_checks(timetabled_route, possible_ends, stops, &visited_route_ids)?;
    println!("{}Found suitable routes: {:?}", indent, new_to_check);
    let mut best_closest = std::f64::INFINITY;
    let mut best_route = None;

    for (new_start, (end_stop, route_idx), closest_app, stop_indexes) in new_to_check {
        let route = &possible_ends.get(end_stop).unwrap()[route_idx];
        let mut new_visited_routes = visited_route_ids.clone();
        new_visited_routes.insert(route.id.clone());
        println!(
            "{}Checking route: {:?}",
            indent,
            (new_start, end_stop, route_idx)
        );
        let (sub_closest, mut sub_best_route, mut sub_stop_idxs) = match do_find_route(
            ("  ".to_owned() + &indent).to_string(),
            &timetabled_route[new_start..],
            possible_routes,
            stops,
            new_visited_routes,
        ) {
            Err(e) => {
                println!("{}Failed to find route starting at {}: {}", indent, new_start, e);
                continue;
            }
            Ok(value) => value,
        };
        if (closest_app + sub_closest) < best_closest {
            best_closest = closest_app + sub_closest;
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
        HashSet::new(),
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
        for idx in stop_idx_segment {
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
