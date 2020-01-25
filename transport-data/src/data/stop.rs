use crate::shared::FareZone;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{collections::BTreeMap, error::Error, fs::File, path::Path};

pub fn load_stops(data_folder: &Path) -> Result<StopList, Box<dyn Error>> {
    let stops = from_reader(File::open(data_folder.join("stops.json"))?)?;
    Ok(StopList::new(stops))
}

pub struct StopList {
    pub stop_by_sms: BTreeMap<String, Stop>,
}

impl StopList {
    pub fn new(stops: Vec<Stop>) -> Self {
        Self {
            stop_by_sms: stops
                .into_iter()
                .map(|stop| (stop.sms.clone(), stop))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stop {
    name: String,
    sms: String,
    farezone: FareZone,
    lat: BigDecimal,
    lon: BigDecimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;
    use std::str::FromStr;

    #[test]
    fn test_parse_stops() {
        assert_eq!(
            from_str::<Stop>(
                r#"{
                    "farezone": "12/13",
                    "lat": -41.058710,
                    "lon": 175.494751,
                    "name": "Fare Zone Boundary 12/13 Waiohine River Bridge (SH2)",
                    "sms": "0004"
                }"#
            )
            .unwrap(),
            Stop {
                name: "Fare Zone Boundary 12/13 Waiohine River Bridge (SH2)".to_string(),
                sms: "0004".to_string(),
                farezone: FareZone::ZoneBoundry(12, 13),
                lat: BigDecimal::from_str("-41.058710").unwrap(),
                lon: BigDecimal::from_str("175.494751").unwrap(),
            }
        )
    }
}
