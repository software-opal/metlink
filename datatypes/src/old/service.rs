use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub trait ToPoint {
    fn to_lon_lat(&self) -> (f64, f64);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub code: String,
    pub last_modified: DateTime<FixedOffset>,
    pub mode: String,
    pub name: String,
    pub routes: Vec<Route>,
    pub stops: Vec<Stop>,
}

impl Service {
    pub fn regenerate_routes(&self) -> Option<Self> {
        let mut new_routes = Vec::with_capacity(self.routes.len());
        let stop_points = self
            .stops
            .iter()
            .map(|s| (s.to_lon_lat(), &s.sms))
            .collect::<Vec<_>>();

        for route in &self.routes {
            new_routes.push(route.regenerate_route(&stop_points)?);
        }
        Some(Service {
            routes: new_routes,
            ..self.clone()
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Route {
    pub route: Vec<RoutePoint>,
    pub stops: Vec<String>,
}

impl Route {
    pub fn regenerate_route(&self, all_stops: &[((f64, f64), &String)]) -> Option<Self> {
        let points = self
            .route
            .iter()
            .map(|p| p.to_lon_lat())
            .collect::<Vec<_>>();
        let (first, points) = points.split_first()?;
        let (last, points) = points.split_last()?;

        for (i, window) in points.windows(3).enumerate() {
            let dist_lon = (window[0].0 - window[2].0).abs();
            let dist_lat = (window[0].1 - window[2].1).abs();
            if dist_lon < 1e-5
dist_lat < 1e-5 {
                println!("Stop at {}, {:?}", i, window[1]);
            }
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum RoutePoint {
    Stop { lat: f64, lon: f64, stop: String },
    Intermediate { lat: f64, lon: f64 },
}

impl RoutePoint {
    fn to_intermediate(&self) -> Self {
        let (lon, lat) = self.to_lon_lat();
        RoutePoint::Intermediate { lat, lon }
    }
    fn with_stop(&self, stop: String) -> Self {
        let (lon, lat) = self.to_lon_lat();
        RoutePoint::Stop { lat, lon, stop }
    }
}

impl ToPoint for RoutePoint {
    fn to_lon_lat(&self) -> (f64, f64) {
        match self {
            RoutePoint::Stop { lon, lat, stop: _ } => (*lon, *lat),
            RoutePoint::Intermediate { lon, lat } => (*lon, *lat),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stop {
    pub lat: f64,
    pub lon: f64,
    pub farezone: String,
    pub name: String,
    pub sms: String,
}
impl ToPoint for Stop {
    fn to_lon_lat(&self) -> (f64, f64) {
        (self.lon, self.lat)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Timetable {
    pub day: NaiveDate,
    pub direction: Direction,
    pub service: String,
    pub timetables: Vec<TimetabledService>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Direction {
    #[serde(rename = "inbound")]
    Inbound,
    #[serde(rename = "outbound")]
    Outbound,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimetabledService {
    pub stops: Vec<String>,
    pub times: Vec<DateTime<FixedOffset>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_name() {
        let svc: Service = serde_json::from_str(include_str!("testdata/route-264.json")).unwrap();
        assert_eq!(
            svc.routes[0].stops,
            vec![String::from("1105"), String::from("1002")]
        );
        let new_svc = svc.regenerate_routes().unwrap();
        assert_eq!(
            svc.routes[0].stops,
            vec![
                String::from("1105"),
                String::from("1194"),
                String::from("1000"),
                String::from("1002")
            ]
        );
    }
}
