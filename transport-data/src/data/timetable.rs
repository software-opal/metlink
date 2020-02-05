use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{ fs::File, path::Path};
use anyhow::{Context, Error, Result};

use crate::data::files::{timetable_json, timetables_json};

pub fn load_timetable<S: Into<String>>(
    data_folder: &Path,
    service_code: S,
    date: NaiveDate,
    direction: RouteDirection,
) -> Result<Vec<Timetable>> {
    let services = from_reader(File::open(timetable_json(
        data_folder,
        service_code,
        date,
        direction,
    ))?)?;
    Ok(services)
}

pub fn load_timetables<S: Into<String>>(
    data_folder: &Path,
    service_code: S,
) -> Result<$1> {
    let timetables = from_reader(File::open(timetables_json(data_folder, service_code))?)?;
    Ok(timetables)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Timetable {
    day: NaiveDate,
    direction: RouteDirection,
    service: String,
    timetables: Vec<TimetabledRoute>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimetabledRoute {
    stops: Vec<String>,
    times: Vec<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RouteDirection {
    #[serde(rename = "inbound")]
    Inbound,
    #[serde(rename = "outbound")]
    Outbound,
}
impl RouteDirection {
    pub fn name(&self) -> String {
        match self {
            Self::Inbound => "inbound".to_string(),
            Self::Outbound => "outbound".to_string(),
        }
    }
    pub fn from_name(name: &dyn ToString) -> Option<Self> {
        match name.to_string().to_lowercase().trim() {
            "inbound" => Some(Self::Inbound),
            "outbound" => Some(Self::Outbound),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::from_str;

    const TZ_PLUS_12: i32 = 12 * 3600;

    #[test]
    fn test_name() {
        assert_eq!(
            from_str::<Timetable>(
                r#"{
                "day": "2019-04-13",
                "direction": "inbound",
                "service": "WHF",
                "timetables": [
                    {
                        "stops": [
                            "9997",
                            "9998",
                            "9999",
                            "9998",
                            "9997"
                        ],
                        "times": [
                            "2019-04-13T10:00:00+12:00",
                            "2019-04-13T10:25:00+12:00",
                            "2019-04-13T10:40:00+12:00",
                            "2019-04-13T10:50:00+12:00",
                            "2019-04-13T11:15:00+12:00"
                        ]
                    }
                ]
                }"#
            )
            .unwrap(),
            Timetable {
                day: NaiveDate::from_ymd(2019, 04, 13),
                direction: RouteDirection::Inbound,
                service: "WHF".to_string(),
                timetables: vec![TimetabledRoute {
                    stops: vec![
                        "9997".to_string(),
                        "9998".to_string(),
                        "9999".to_string(),
                        "9998".to_string(),
                        "9997".to_string()
                    ],
                    times: vec![
                        FixedOffset::east(TZ_PLUS_12)
                            .ymd(2019, 04, 13)
                            .and_hms(10, 0, 0),
                        FixedOffset::east(TZ_PLUS_12)
                            .ymd(2019, 04, 13)
                            .and_hms(10, 25, 0),
                        FixedOffset::east(TZ_PLUS_12)
                            .ymd(2019, 04, 13)
                            .and_hms(10, 40, 0),
                        FixedOffset::east(TZ_PLUS_12)
                            .ymd(2019, 04, 13)
                            .and_hms(10, 50, 0),
                        FixedOffset::east(TZ_PLUS_12)
                            .ymd(2019, 04, 13)
                            .and_hms(11, 15, 0),
                    ]
                }]
            }
        )
    }
}
