use super::*;
use metlink_transport_data::data::{ExtendedService, Stop};
use serde_json::from_str;

const STOPS_JSON: &'static str = include_str!("./stops.json");
fn stops() -> StopList {
    let stops: Vec<Stop> = from_str(STOPS_JSON).unwrap();
    stops
        .into_iter()
        .map(|stop| (stop.sms.clone(), stop))
        .collect()
}
fn service(json: &str) -> ExtendedService {
    from_str(json).unwrap()
}

#[test]
fn test_service_236() {
    let stops = stops();
    let routes = organise_routes(service(include_str!("./236-service.json")));
    let failing_route = vec![
        // 1st segment
        "2738", "2740", "2744", "2748", "2750", "2752", "2754", "2756", "2758", "2760", "2761",
        "2762", "2764", "2766", "2768", "2770", "2566", "2568",
        // 2nd segment
        "2570", "2604", "2600", "2602", "2572",

        "2574", "2576", "2578", "2580", "2582", "2584", "2586", "2588", "2590", "2592",
        "2594", "2596", "2031", "2026", "2028", "2030", "2002",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    let result = find_route(&failing_route, &routes, &stops);
    let (segments, _route) = result.unwrap();
    let segment_ids = segments.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(segment_ids, vec!["0"]);
}
