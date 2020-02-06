use crate::{data::files::stops_json, shared::FareZone};
use anyhow::{Context, Error, Result};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{fs::File, path::Path};

pub fn load_stops(data_folder: &Path) -> Result<Vec<Stop>> {
    let stops =
        from_reader(File::open(stops_json(data_folder)).context("Failed to open stops file")?)
            .context("Failed to read stops file")?;
    Ok(stops)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stop {
    pub name: String,
    pub sms: String,
    pub farezone: FareZone,
    pub lat: BigDecimal,
    pub lon: BigDecimal,
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
