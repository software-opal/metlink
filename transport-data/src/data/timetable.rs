use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{error::Error, fs::File, path::Path};

pub fn load_timetable(
    data_folder: &Path,
    service_code: String,
    date: NaiveDate,
    direction: RouteDirection,
) -> Result<Vec<Timetable>, Box<dyn Error>> {
    let services = from_reader(File::open(data_folder.join("stops.json"))?)?;
    Ok(services)
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RouteDirection {
    Inbound,
    Outbound,
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
