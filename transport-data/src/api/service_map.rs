use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::shared::ServiceMode;

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

#[derive(Debug, Clone, PartialEq)]
pub struct StringedLatLong {
    lat: BigDecimal,
    long: BigDecimal,
    extra: bool,
}

impl StringedLatLong {
    fn to_metlink_str(&self) -> String {
        if self.extra {
            format!("{},{},0", self.lat, self.long)
        } else {
            format!("{},{}", self.lat, self.long)
        }
    }

    fn from_metlink_str(s: &str) -> Result<Self, StringedLatLongError> {
        if s.is_empty() {
            return Err(StringedLatLongError::EmptyString);
        }
        let mut parts = s.split(',');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(lat), Some(long), extra) => {
                let extra = match extra {
                    None => false,
                    Some("0") => true,
                    _ => return Err(StringedLatLongError::InvalidString),
                };
                match (lat.parse::<BigDecimal>(), long.parse::<BigDecimal>()) {
                    (Ok(lat), Ok(long)) => Ok(StringedLatLong { lat, long, extra }),
                    _ => Err(StringedLatLongError::NotAnNumber),
                }
            }
            _ => Err(StringedLatLongError::InvalidString),
        }
    }
}

impl Serialize for StringedLatLong {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_metlink_str())
    }
}
impl<'de> Deserialize<'de> for StringedLatLong {
    fn deserialize<D>(deserializer: D) -> Result<StringedLatLong, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ServiceModeVisitor;
        impl<'de> Visitor<'de> for ServiceModeVisitor {
            type Value = StringedLatLong;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a String that represents a StringedLatLong")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                StringedLatLong::from_metlink_str(v).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(ServiceModeVisitor)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StringedLatLongError {
    EmptyString,
    NotAnNumber,
    InvalidString,
}

impl fmt::Display for StringedLatLongError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyString => write!(f, "EmptyString"),
            Self::NotAnNumber => write!(f, "NotAnNumber"),
            Self::InvalidString => write!(f, "InvalidString"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{assert_tokens, Token};
    #[test]
    fn test_fare_zone_serde() {
        assert_tokens(
            &StringedLatLong {
                lat: BigDecimal::from(5),
                long: BigDecimal::from(-2),
                extra: false,
            },
            &[Token::String("5,-2")],
        );
        assert_tokens(
            &StringedLatLong {
                lat: BigDecimal::from(5),
                long: BigDecimal::from(-2),
                extra: true,
            },
            &[Token::String("5,-2,0")],
        );
    }
}
