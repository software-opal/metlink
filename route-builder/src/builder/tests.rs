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
    // Stops were incorrectly detected in the python, causing it to not have a segment where it should have
    let stops = stops();
    let routes = organise_routes(service(include_str!("./236-service.json")));
    let failing_route = vec![
        // 1st segment
        "2738", "2740", "2744", "2748", "2750", "2752", "2754", "2756", "2758", "2760", "2761",
        "2762", "2764", "2766", "2768", "2770", "2566", "2568", // 2nd segment
        "2570", "2604", "2600", "2602", "2572", "2574", "2576", "2578", "2580", "2582", "2584",
        "2586", "2588", "2590", "2592", "2594", "2596", "2031", "2026", "2028", "2030", "2002",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    let result = find_route(&failing_route, &routes, &stops);
    let (segments, _route) = result.unwrap();
    let segment_ids = segments.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(
        segment_ids,
        vec![
            "2738-2570",
            "2570-2604",
            "2604-2600",
            "2600-2602",
            "2602-2572",
            "2572-2596",
            "2596-2026",
            "2026-2002"
        ]
    );
}

#[test]
fn test_service_30x() {
    // Failed to take into account the distance between the start/end stops in the distance calculation
    // This meant it picked some segments that were in the wrong place
    let stops = stops();
    let routes = organise_routes(service(include_str!("./30x-service.json")));
    let failing_route = vec![
        "6001", "5502", "5506", "5508", "5510", "5513", "5514", "5516", "6080", "6081", "7232",
        "6086", "6032", "6033", "6034", "6035", "6036", "6037", "6038", "6039", "6040", "6062",
        "6064", "6065", "6066", "6067", "6068", "6069", "6070", "6071", "7072",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    let result = find_route(&failing_route, &routes, &stops);
    let (segments, _route) = result.unwrap();
    let segment_ids = segments.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(segment_ids, vec!["6001-6040", "6040-7072"]);
}

#[test]
fn test_service_854() {
    // Visits the same bit of route twice
    let stops = stops();
    let routes = organise_routes(service(include_str!("./854-service.json")));
    let failing_route = vec![
        "9135", "9136", "8165", "8164", "8163", "9197", "9158", "9199", "8112", "8111", "8110",
        "8009", "8008", "8007", "8006", "8005", "8004", "8003", "8002", "9001", "9090", "9092",
        "9093", "8099", "8094", "8098", "8097", "8096", "8093", "8091", "8092", "8090", "8003",
        "8002", "9001",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    let result = find_route(&failing_route, &routes, &stops);
    let (segments, _route) = result.unwrap();
    let segment_ids = segments.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(
        segment_ids,
        vec![
            "9135-8163",
            "8163-8003",
            "8003-9001",
            "9001-8003",
            "8003-9001"
        ]
    );
}

#[test]
fn test_service_ccl() {
    // Cable car
    // Sanity check did not account for routes with stops at sequential points.
    let stops = stops();
    let routes = organise_routes(service(include_str!("./CCL-service.json")));
    let failing_route = vec!["KELB", "SALA", "TALA", "CLIF", "LAMB"]
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let result = find_route(&failing_route, &routes, &stops);
    let (segments, _route) = result.unwrap();
    let segment_ids = segments.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(
        segment_ids,
        vec![
        "KELB-LAMB"
        ]
    );
}
