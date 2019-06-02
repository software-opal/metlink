pub use crate::py_data::timetables::Direction;
use crate::py_data::timetables::Timetable;
use crate::py_data::timetables::TimetabledService;
use chrono::DateTime;
use chrono::FixedOffset;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceTimetable {
    pub code: String,
    pub entries: Vec<ServiceTimetableEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceTimetableEntry {
    pub direction: Direction,
    pub stops: Vec<String>,
    pub times: Vec<DateTime<FixedOffset>>,
}

impl ServiceTimetable {
    pub fn from_py_timetables(ts: Vec<Timetable>) -> Vec<ServiceTimetable> {
        let mut timetables_by_code: BTreeMap<String, Vec<Timetable>> = BTreeMap::new();
        ts.into_iter().for_each(|t| {
            timetables_by_code
                .entry(t.service.clone())
                .or_default()
                .push(t)
        });
        timetables_by_code
            .into_iter()
            .map(|(code, ts)| {
                let entries =
                    ts.into_iter()
                        .flat_map(|t| {
                            let direction = t.direction;
                            t.timetables.into_iter().map(
                                move |TimetabledService { stops, times }| ServiceTimetableEntry {
                                    stops,
                                    times,
                                    direction: direction.clone(),
                                },
                            )
                        })
                        .collect::<Vec<_>>();
                ServiceTimetable { code, entries }
            })
            .collect::<Vec<_>>()
    }
}
