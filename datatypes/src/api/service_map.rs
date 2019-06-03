use super::types::{ServiceMode, StringedLatLong};
use chrono::{DateTime, FixedOffset};
use geo::Point;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceMap {
    /*
    {
        "Code":"20",
        "Name":"Kilbirnie - Mt Victoria - Courtenay Place",
        "Mode":"Bus",
        "LastModified":"2019-03-29T15:11:08+13:00",
        "StopLocations": [...],
        "RouteMaps":[...],
        "TrimmedCode":"20",
        "Link":"\/timetables\/bus\/20"
    }
    */
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Mode")]
    pub mode: ServiceMode,
    #[serde(rename = "LastModified")]
    pub last_modified: DateTime<FixedOffset>,
    #[serde(rename = "StopLocations")]
    pub stop_locations: Vec<MapStop>,
    #[serde(rename = "RouteMaps")]
    pub route_maps: Vec<RouteMap>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapStop {
    #[serde(rename = "Sms")]
    pub sms: String,
    #[serde(rename = "LatLng")]
    pub location: StringedLatLong,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteMap {
    #[serde(rename = "Path")]
    pub path: Vec<StringedLatLong>,
}

impl RouteMap {
    pub fn clone_to_point_list(&self) -> Vec<Point<f64>> {
        self.path.iter().map(|p| p.into()).collect::<Vec<_>>()
    }
}
