use super::types::ServiceMode;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    /*
    {
        "Code":"1",
        "Name":"Johnsonville West\/Churton Park\/Grenada Village - Island Bay",
        "Mode":"Bus",
        "LastModified":"2019-02-22T10:43:01+13:00",
        "TrimmedCode":"1",
        "Link":"\/timetables\/bus\/1",
        "AliasNames": "Newtown School, St Anne's Primary School, ..."
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
}

impl PartialEq for Service {
    fn eq(&self, rhs: &Service) -> bool {
        return self.code == rhs.code;
    }
}

impl Service {
    // ...
}
