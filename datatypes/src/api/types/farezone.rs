use serde::de::Visitor;
use serde::Deserializer;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum FareZone {
    Zone(u8),
    ZoneBoundry(u8, u8),
    NotZoned,
}

impl FareZone {
    fn to_metlink_str(&self) -> String {
        match self {
            FareZone::NotZoned => "".to_string(),
            FareZone::Zone(zone) => zone.to_string(),
            FareZone::ZoneBoundry(inner_zone, outer_zone) => {
                if inner_zone % 2 == 0 {
                    format!("{}/{}", inner_zone, outer_zone)
                } else {
                    format!("{}/{}", outer_zone, inner_zone)
                }
            }
        }
    }
}

impl Display for FareZone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FareZone::NotZoned => write!(f, "Not covered by fare zones"),
            FareZone::Zone(zone) => write!(f, "{}", zone),
            FareZone::ZoneBoundry(inner_zone, outer_zone) => {
                write!(f, "{}/{}", inner_zone, outer_zone)
            }
        }
    }
}

impl FromStr for FareZone {
    type Err = ParseFareZoneError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(FareZone::NotZoned);
        }
        let mut parts = s.split('/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(zone), None, _) => match zone.parse::<u8>() {
                Ok(zone) => Ok(FareZone::Zone(zone)),
                Err(e) => Err(ParseFareZoneError {
                    kind: FareZoneErrorKind::NotAnInteger,
                    inner: Some(e),
                }),
            },
            (Some(left_zone), Some(right_zone), None) => {
                match (left_zone.parse::<u8>(), right_zone.parse::<u8>()) {
                    (Ok(left_zone), Ok(right_zone)) => {
                        // Metlink encodes zone boundaries with the even number on the left. We standardise.
                        if left_zone < right_zone {
                            Ok(FareZone::ZoneBoundry(left_zone, right_zone))
                        } else {
                            Ok(FareZone::ZoneBoundry(right_zone, left_zone))
                        }
                    }
                    (Err(e), _) => Err(ParseFareZoneError {
                        kind: FareZoneErrorKind::NotAnInteger,
                        inner: Some(e),
                    }),
                    (_, Err(e)) => Err(ParseFareZoneError {
                        kind: FareZoneErrorKind::NotAnInteger,
                        inner: Some(e),
                    }),
                }
            }
            _ => Err(ParseFareZoneError {
                kind: FareZoneErrorKind::InvalidString,
                inner: None,
            }),
        }
    }
}

impl Serialize for FareZone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_metlink_str())
    }
}
impl<'de> Deserialize<'de> for FareZone {
    fn deserialize<D>(deserializer: D) -> Result<FareZone, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ServiceModeVisitor;
        impl<'de> Visitor<'de> for ServiceModeVisitor {
            type Value = FareZone;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a String that represents a FareZone")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FareZone::from_str(v).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(ServiceModeVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseFareZoneError {
    kind: FareZoneErrorKind,
    inner: Option<ParseIntError>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FareZoneErrorKind {
    NotAnInteger,
    InvalidString,
}

impl Display for ParseFareZoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.kind, &self.inner) {
            (FareZoneErrorKind::InvalidString, _) => write!(f, "fare zone not in expected form"),
            (FareZoneErrorKind::NotAnInteger, None) => {
                write!(f, "fare zone contains an invalid numeric component")
            }
            (FareZoneErrorKind::NotAnInteger, Some(e)) => {
                write!(f, "fare zone contains an invalid numeric component: {}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_fare_zone_serde() {
        assert_tokens(&FareZone::NotZoned, &[Token::String("")]);
        assert_tokens(&FareZone::Zone(10), &[Token::String("10")]);
        assert_tokens(&FareZone::Zone(0), &[Token::String("0")]);
        assert_tokens(&FareZone::ZoneBoundry(2, 3), &[Token::String("2/3")]);
        assert_tokens(&FareZone::ZoneBoundry(3, 4), &[Token::String("4/3")]);
    }
}
