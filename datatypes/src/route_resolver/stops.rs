use std::collections::BTreeMap;

pub fn generate_routes_for(
    route: &[String],
    map_start_to_ends: &BTreeMap<String, BTreeMap<String, Vec<usize>>>,
) -> Vec<Vec<usize>> {
    let start = match route.first() {
        Some(s) => s,
        None => return vec![],
    };
    let ends = match map_start_to_ends.get(start.as_str()) {
        Some(e) => e,
        None => return vec![],
    };
    let mut current_routes = vec![];
    for (end, route_segments) in ends {
        match route.iter().position(|v| v == end) {
            Some(0) => continue,
            Some(pos) => {
                let partial_routes = if pos == (route.len() - 1) {
                    vec![vec![]]
                } else {
                    generate_routes_for(&route[pos..], map_start_to_ends)
                };
                current_routes.reserve(partial_routes.len() * route_segments.len());
                for &segment in route_segments {
                    for partial_route in partial_routes.iter() {
                        let mut new_route = Vec::with_capacity(partial_route.len() + 1);
                        new_route.push(segment);
                        new_route.extend(partial_route.iter().map(|&v| v));
                        current_routes.push(new_route);
                    }
                }
            }
            None => continue,
        }
    }
    return current_routes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simples() {
        let mut map_start_to_ends = BTreeMap::new();
        map_start_to_ends.insert(
            "123".to_string(),
            vec![
                ("124".to_string(), vec![0usize, 2]),
                ("126".to_string(), vec![1usize]),
            ]
            .into_iter()
            .collect(),
        );

        let routes = generate_routes_for(
            &[
                "123".to_string(),
                "124".to_string(),
                "125".to_string(),
                "126".to_string(),
            ],
            &map_start_to_ends,
        );
        assert_eq!(routes, vec![vec![1]])
    }
}
