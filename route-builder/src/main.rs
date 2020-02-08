use crate::{builder::*, data_utils::*, utils::*};
use anyhow::{Context, Result};
use chrono::Local;
use metlink_transport_data::data::save_routes;
use std::path::Path;

mod builder;
mod data_utils;
mod utils;

fn main() -> Result<()> {
    // timeit("Total Time: ", || {
    let today = Local::now().date().naive_local();
    let folder = Path::new("./data/");
    let (stops, services) = timeit("Load Time", || {
        join_results(
            || load_stops(folder),
            || load_services(folder),
        )
    })?;
    services
        .iter()
        .skip(40)
        .try_for_each::<_, Result<()>>(|(_, service)| {
            let (ext_service, timetables) = join_results(
                || {
                    timeit(format!("load_ext_service({:?})", &service.code), || {
                        load_ext_service(folder, &service.code)
                    })
                },
                || {
                    timeit(format!("load_timetable({:?})", &service.code), || {
                        load_timetable(folder, &service.code)
                    })
                },
            )
            .with_context(|| format!("Could not load information for {:?}", service))?;
            let (timetables, routes) = rayon::join(
                || {
                    timeit(format!("organise_timetables({:?})", &service.code), || {
                        organise_timetables(timetables, Some(today))
                    })
                },
                || {
                    timeit(format!("organise_routes({:?})", &service.code), || {
                        organise_routes(ext_service)
                    })
                },
            );
            let mapped_routes = timeit(format!("find_route({:?})", service.code), || {
                collect_results(
                    timetables
                        .iter()
                        .map(|(timetable, times)| {
                            let (_, route) =
                                find_route(&timetable, &routes, &stops).with_context(|| {
                                    format!(
                                        "Failed to find route. This route exists on: {:?}",
                                        &times[..5]
                                    )
                                })?;
                            Ok(route)
                        })
                        .collect::<Vec<_>>(),
                )
            })?;
            save_routes(folder, &service.code, &mapped_routes)?;
            Ok(())
        })?;

    Ok(())
    // })?;
    // Ok(())
}
