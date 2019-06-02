use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

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
}
impl FromStr for ServiceMode {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Bus" => ServiceMode::Bus,
            "Train" => ServiceMode::Train,
            "Ferry" => ServiceMode::Ferry,
            "Cable Car" => ServiceMode::CableCar,
            other => ServiceMode::Other(other.to_owned()),
        })
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
                Ok(match ServiceMode::from_str(v) {
                    Ok(sm) => sm,
                    Err(_) => unreachable!(),
                })
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
}
