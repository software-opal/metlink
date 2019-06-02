use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Timetable {
    pub day: NaiveDate,
    pub direction: Direction,
    pub service: String,
    pub timetables: Vec<TimetabledService>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
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
