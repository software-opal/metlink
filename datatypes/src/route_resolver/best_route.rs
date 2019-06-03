
use crate::api::service_map::RouteMap;
use crate::utils::{StartEndRouteMap, StopId};
use std::collections::BTreeMap;


fn reverse_map_start_to_end(
    map_start_to_ends: &StartEndRouteMap,
) -> BTreeMap<usize, (&StopId, &StopId)> {
    map_start_to_ends
        .iter()
        .flat_map(|(start, ends_to_route)| {
            ends_to_route
                .iter()
                .flat_map(move |(end, route_ids)| route_ids.iter().map(move |i| (*i, (start, end))))
        })
        .collect::<BTreeMap<_, _>>()
}

pub fn remove_common_elements(
    route_segments: &[Vec<usize>],
) -> (Vec<usize>, Vec<Vec<usize>>, Vec<usize>) {
    let mut prefix = Vec::new();
    let mut postfix = Vec::new();

    let min_len = route_segments.iter().map(|arr| arr.len()).min().unwrap();
    let (first, rest) = route_segments.split_first().unwrap();

    // Remove all the common starting elements.
    for (i, &val) in first.iter().enumerate().take(min_len) {
        if rest.iter().all(|r| r[i] == val) {
            prefix.push(val)
        } else {
            break;
        }
    }
    // Then remove all the common trailing elements, up to where we finished checking for the prefixed length.
    for (i, &val) in first.iter().rev().take(min_len - prefix.len()).enumerate() {
        if rest.iter().all(|r| r[r.len() - 1 - i] == val) {
            postfix.push(val)
        }
    }
    // Flip the postfix so that it matches the order in the segments
    postfix.reverse();

    let subseg_start = prefix.len();
    let subseg_end_offset = postfix.len();
    let new_subsegments = route_segments
        .iter()
        .map(|s| {
            let slen = s.len();
            let subseg_end = slen - subseg_end_offset;
            assert!(subseg_start <= slen);
            assert_eq!(s[0..subseg_start], prefix[..]);
            assert!(subseg_end_offset <= slen);
            assert_eq!(s[subseg_end..slen], postfix[..]);
            s[subseg_start..subseg_end].to_vec()
        })
        .collect::<Vec<_>>();

    (prefix, new_subsegments, postfix)
}

pub fn prune_common_segments_from_route(
    orig_route: &[StopId],
    ids_to_start_end: BTreeMap<usize, (&StopId, &StopId)>,
    prefix: Vec<usize>,
    postfix: Vec<usize>,
) -> Vec<StopId> {
    let mut route = orig_route.to_vec();
    // Remove the common segments from the start and end of the routes.
    for idx in prefix {
        let (start, end) = ids_to_start_end[&idx];
        assert_eq!(&route[0], start);
        let end_idx = route.iter().position(|v| v == end).unwrap();
        route = route.split_off(end_idx);
        if route.len() == 1 {
            route = Vec::new();
        } else {
            assert_eq!(&route[0], end);
        }
    }
    for idx in postfix.iter().rev() {
        let (start, end) = ids_to_start_end[&idx];
        println!("Start: {}; End: {}", start, end);
        assert_eq!(&route.last(), &Some(end));
        let start_idx = route.iter().rposition(|v| v == start).unwrap();
        route.split_off(start_idx + 1);
        if route.len() == 1 {
            route = Vec::new();
        } else {
            assert_eq!(&route.last(), &Some(start));
        }
    }
    route
}

pub fn choose_best_route(
    stop_locations: Vec<Point<f64>>,
    possible_routes: Vec<Vec<Point<f64>>>,
) -> Option<usize> {

    assert!(false);
    None
}


pub fn find_best_route(
    orig_route: &[StopId],
    route_segments: &[Vec<usize>],
    map_start_to_ends: &StartEndRouteMap,
    stop_locations: &BTreeMap<StopId, Point<f64>>,
    route_maps: &[RouteMap],
) -> Option<usize> {
    if route_segments.is_empty() {
        return None;
    } else if route_segments.len() == 1 {
        return Some(0);
    }

    let ids_to_start_end = reverse_map_start_to_end(map_start_to_ends);

    println!("{:?}", route_segments);
    println!("{:?}", ids_to_start_end);
    let (prefix, deduped_segments, postfix) = remove_common_elements(route_segments);
    println!("{:?} | {:?} | {:?}", prefix, deduped_segments, postfix);

    let route = prune_common_segments_from_route(orig_route, ids_to_start_end, prefix, postfix);
    let route_stop_locations = route
        .iter()
        .map(|sms| stop_locations[sms].into())
        .collect::<Vec<Point<_>>>();

    let route_paths = route_segments
        .iter()
        .map(|path_segments| {
            path_segments
                .iter()
                .flat_map(|&i| route_maps[i].path.iter().cloned().map(|p| p.into()))
                .collect::<Vec<Point<_>>>()
        })
        .collect::<Vec<_>>();
    println!("{:?}", route_paths);
    choose_best_route(route_stop_locations, route_paths)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_common_elements() {
        let elms = [
            vec![0, 1, 2, 3],
            vec![0, 4, 5, 3],
            vec![3, 4, 5],
            vec![0, 4, 5],
        ];
        assert_eq!(
            remove_common_elements(&elms[0..1]),
            (vec![0, 1, 2, 3], vec![vec![]], vec![])
        );
        assert_eq!(
            remove_common_elements(&elms[0..=1]),
            (vec![0], vec![vec![1, 2], vec![4, 5]], vec![3])
        );
        assert_eq!(
            remove_common_elements(&elms[1..=2]),
            (vec![], vec![vec![0, 4, 5, 3], vec![3, 4, 5]], vec![])
        );
        assert_eq!(
            remove_common_elements(&elms[2..=3]),
            (vec![], vec![vec![3], vec![0]], vec![4, 5])
        );
    }
}
