use crate::{data::files::stops_json, shared::FareZone};
use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, ToPrimitive};
use geojson::{feature, Feature, Geometry, Position, Value};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, Map};
use std::{fs::File, io::BufReader, path::Path};

pub fn load_stops(data_folder: &Path) -> Result<Vec<Stop>> {
    let stops = from_reader(BufReader::new(
        File::open(stops_json(data_folder)).context("Failed to open stops file")?,
    ))
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

impl Into<Feature> for &Stop {
    fn into(self) -> Feature {
        let mut props = Map::new();
        props.insert("name".to_string(), self.name.clone().into());
        props.insert("sms".to_string(), self.sms.clone().into());
        props.insert(
            "farezone".to_string(),
            self.farezone.clone().to_string().into(),
        );
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Point(self.into()))),
            id: Some(feature::Id::String(self.sms.clone())),
            properties: Some(props),
            foreign_members: None,
        }
    }
}
impl Into<Position> for &Stop {
    fn into(self) -> Position {
        vec![self.lon.to_f64().unwrap(), self.lat.to_f64().unwrap()]
    }
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
