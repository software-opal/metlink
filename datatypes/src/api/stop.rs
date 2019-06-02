use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use super::types::FareZone;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StopListResponse {
    #[serde(rename = "LastModified")]
    pub last_modified: DateTime<FixedOffset>,
    #[serde(rename = "Stops")]
    pub stops: Vec<Stop>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stop {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Sms")]
    sms: String,
    #[serde(rename = "Farezone")]
    farezone: FareZone,
    #[serde(rename = "Lat")]
    lat: BigDecimal,
    #[serde(rename = "Long")]
    long: BigDecimal,
    #[serde(rename = "LastModified")]
    last_modified: DateTime<FixedOffset>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::TimeZone;
    use serde_json::from_str;
    use std::str::FromStr;

    const TZ_PLUS_12: i32 = 12 * 3600;

    #[test]
    fn test_name() {
        assert_eq!(
            from_str::<Stop>(
                r#"{
                    "Name": "Paremata Station",
                    "Sms": "2600",
                    "Farezone": "6\/5",
                    "Lat": "-41.10615",
                    "Long": "174.8667714",
                    "LastModified": "2019-04-16T00:00:31+12:00"
                }"#
            )
            .unwrap(),
            Stop {
                name: "Paremata Station".to_string(),
                sms: "2600".to_string(),
                farezone: FareZone::ZoneBoundry(5, 6),
                lat: BigDecimal::from_str("-41.10615").unwrap(),
                long: BigDecimal::from_str("174.8667714").unwrap(),
                last_modified: FixedOffset::east(TZ_PLUS_12).timestamp(1555329631, 0),
            }
        )
    }
}
