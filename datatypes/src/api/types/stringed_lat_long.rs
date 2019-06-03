use bigdecimal::BigDecimal;
use geo::Point;
use num_traits::cast::ToPrimitive;
use serde::de::Visitor;
use serde::Deserializer;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

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
}

impl Display for StringedLatLong {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.lat, self.long)
    }
}

impl From<StringedLatLong> for Point<f64> {
    fn from(point: StringedLatLong) -> Point<f64> {
        Point::new(point.lat.to_f64().unwrap(), point.long.to_f64().unwrap())
    }
}
impl From<&StringedLatLong> for Point<f64> {
    fn from(point: &StringedLatLong) -> Point<f64> {
        Point::new(point.lat.to_f64().unwrap(), point.long.to_f64().unwrap())
    }
}

impl FromStr for StringedLatLong {
    type Err = ParseStringedLatLongError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseStringedLatLongError {
                kind: StringedLatLongErrorKind::EmptyString,
            });
        }
        let mut parts = s.split(',');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(lat), Some(long), extra) => {
                let extra = match extra {
                    None => false,
                    Some("0") => true,
                    _ => {
                        return Err(ParseStringedLatLongError {
                            kind: StringedLatLongErrorKind::InvalidString,
                        })
                    }
                };
                match (lat.parse::<BigDecimal>(), long.parse::<BigDecimal>()) {
                    (Ok(lat), Ok(long)) => Ok(StringedLatLong { lat, long, extra }),
                    _ => Err(ParseStringedLatLongError {
                        kind: StringedLatLongErrorKind::NotAnNumber,
                    }),
                }
            }
            _ => Err(ParseStringedLatLongError {
                kind: StringedLatLongErrorKind::InvalidString,
            }),
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
                StringedLatLong::from_str(v).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(ServiceModeVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseStringedLatLongError {
    kind: StringedLatLongErrorKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StringedLatLongErrorKind {
    EmptyString,
    NotAnNumber,
    InvalidString,
}

impl Display for ParseStringedLatLongError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            StringedLatLongErrorKind::EmptyString => {
                write!(f, "cannot parse lat/long from empty string")
            }
            StringedLatLongErrorKind::NotAnNumber => {
                write!(f, "lat/long contains an invalid decimal component")
            }
            StringedLatLongErrorKind::InvalidString => write!(f, "cannot parse lat/long"),
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
