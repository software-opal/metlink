use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Route {
    end_id: String,
    id: String,
    route: Vec<RouteSegment>,
    start_id: String,
    stops: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RouteSegment {
    lat: BigDecimal,
    lon: BigDecimal,
    stop: Option<String>,
}
