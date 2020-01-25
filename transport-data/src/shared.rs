use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

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
    fn from_metlink_str(s: &str) -> Result<Self, FareZoneError> {
        if s.is_empty() {
            return Ok(FareZone::NotZoned);
        }
        let mut parts = s.split('/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(zone), None, _) => match zone.parse::<u8>() {
                Ok(zone) => Ok(FareZone::Zone(zone)),
                _ => Err(FareZoneError::NotAnInteger),
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
                    _ => Err(FareZoneError::NotAnInteger),
                }
            }
            _ => Err(FareZoneError::InvalidString),
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
                FareZone::from_metlink_str(v).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(ServiceModeVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FareZoneError {
    InvalidString,
    NotAnInteger,
}
impl fmt::Display for FareZoneError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            FareZoneError::InvalidString => fmt.write_str("InvalidString"),
            FareZoneError::NotAnInteger => fmt.write_str("NotAnInteger"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceMode {
    Bus,
    Train,
    Ferry,
    CableCar,
    Other(String),
}
impl ServiceMode {
    fn to_str(&self) -> String {
        match self {
            ServiceMode::Bus => "Bus".to_string(),
            ServiceMode::Train => "Train".to_string(),
            ServiceMode::Ferry => "Ferry".to_string(),
            ServiceMode::CableCar => "Cable Car".to_string(),
            ServiceMode::Other(other) => other.clone(),
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "Bus" => ServiceMode::Bus,
            "Train" => ServiceMode::Train,
            "Ferry" => ServiceMode::Ferry,
            "Cable Car" => ServiceMode::CableCar,
            other => ServiceMode::Other(other.to_owned()),
        }
    }
}

impl Serialize for ServiceMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_str())
    }
}
impl<'de> Deserialize<'de> for ServiceMode {
    fn deserialize<D>(deserializer: D) -> Result<ServiceMode, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ServiceModeVisitor;
        impl<'de> Visitor<'de> for ServiceModeVisitor {
            type Value = ServiceMode;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a String")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ServiceMode::from_str(v))
            }
        }
        deserializer.deserialize_str(ServiceModeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_service_mode_serde() {
        assert_tokens(&ServiceMode::Bus, &[Token::String("Bus")]);
        assert_tokens(&ServiceMode::Train, &[Token::String("Train")]);
        assert_tokens(&ServiceMode::Ferry, &[Token::String("Ferry")]);
        assert_tokens(&ServiceMode::CableCar, &[Token::String("Cable Car")]);
        assert_tokens(
            &ServiceMode::Other("Back of a Moa".to_owned()),
            &[Token::String("Back of a Moa")],
        );
    }
    #[test]
    fn test_fare_zone_serde() {
        assert_tokens(&FareZone::Zone(4), &[Token::String("4")]);
        assert_tokens(&FareZone::ZoneBoundry(5, 6), &[Token::String("6/5")]);
        assert_tokens(&FareZone::NotZoned, &[Token::String("")]);
    }
}
