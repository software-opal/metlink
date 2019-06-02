use crate::service::Service;
use crate::service::TimetabledService;

pub fn get_route_for_timetabled_service(
    ttbl_svc: &TimetabledService,
    service: &Service,
) -> Option<Vec<usize>> {
    let components = service
        .routes
        .iter()
        .map(|r| &r.stops[..])
        .collect::<Vec<&[String]>>();
    find_route_components_used_in_stop_list(&ttbl_svc.stops, &components)
}
use std::fmt::Debug;

pub fn find_route_components_used_in_stop_list<T: Sized + Eq + Debug>(
    stop_list: &[T],
    components: &[&[T]],
) -> Option<Vec<usize>> {
    let mut stop_offset = 0;
    let mut route_components = vec![];
    while (stop_offset + 1) < stop_list.len() {
        let stops = &stop_list[stop_offset..];
        let matching_route_components = components
            .iter()
            .enumerate()
            .inspect(|v| println!("Visiting {:?}", v))
            .filter(|(_, route)| route.len() <= stops.len())
            .inspect(|(_, route)| {
                println!(
                    "Left:  {:?}\nRight: {:?}\nMatch? {:?}",
                    &route[..],
                    &stops[..route.len()],
                    &route[..] == &stops[..route.len()]
                )
            })
            .filter(|(_, route)| route.len() <= stops.len() && &route[..] == &stops[..route.len()])
            .next();
        if let Some((i, route)) = matching_route_components {
            route_components.push(i);
            // We share the end stop with the next segment.
            stop_offset += route.len() - 1;
        } else {
            return None;
        }
    }
    Some(route_components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let components = find_route_components_used_in_stop_list(
            &["a", "b", "c", "d"],
            &[&["b", "a"], &["a", "b"], &["b", "c", "d"], &["b", "d"]],
        );
        assert_eq!(components, Some(vec![1, 2]));
    }

    #[test]
    fn test_real_route_1() {
        let components = find_route_components_used_in_stop_list(
            &[
                "3050", "3052", "3054", "3056", "3058", "3060", "3062", "3064", "3066", "3068",
                "3070", "3072", "3074", "3076", "3078", /* End 12, Start 11*/ "3080",
                /* End 11, Start 8 */ "3081", "3252", "3254", "3256", "3258", "5486", "5488",
                "5490", "5492", "5494", "5496", "5498", "5500", "5502", "5506", "5508", "5510",
                "5513", "5514", "5516",
            ],
            &[
                &[
                    "7158", "7134", "7133", "7132", "7131", "7129", "7128", "7126", "7125", "7124",
                    "7123", "7122", "7121", "7120", "7018", "7017", "7016", "7015", "7014", "7013",
                    "7012", "5000",
                ][..],
                &[
                    "5000", "5002", "5515", "5006", "5008", "5010", "5012", "5014", "5016",
                ],
                &[
                    "5016", "5018", "5020", "5022", "5024", "5025", "5026", "5028", "3260", "3262",
                    "3264", "3266", "3268", "3000",
                ],
                &["3000", "3010"],
                &[
                    "3010", "3270", "3271", "3272", "3273", "3274", "3275", "3276", "3277", "3279",
                ],
                &[
                    "3000", "3400", "3402", "3404", "3406", "3408", "3410", "3412", "3414", "3416",
                    "3418", "3420", "3422", "3424", "3426", "3428", "3430", "3432", "3450", "3434",
                    "3451",
                ],
                &[
                    "3010", "3012", "3014", "3016", "3018", "3020", "3022", "3024", "3026", "3028",
                    "3030", "3034", "3036", "3038", "3040",
                ],
                &[
                    "3451", "3435", "3448", "3452", "3454", "3456", "3458", "3460", "3462", "3464",
                    "3466", "3468", "3470", "3472", "3474", "3476", "3478", "3480", "3482", "3081",
                ],
                &[
                    "3081", "3252", "3254", "3256", "3258", "5486", "5488", "5490", "5492", "5494",
                    "5496", "5498", "5500", "5502", "5506", "5508", "5510", "5513", "5514", "5516",
                ],
                &[
                    "5516", "6012", "6013", "6014", "6015", "6016", "6055", "6119", "6120", "6121",
                    "6123", "6124", "6125", "6126", "6127", "6128", "6129", "6131", "6132", "6133",
                    "6134", "6158",
                ],
                &[
                    "3297", "3299", "3218", "3234", "3236", "3238", "3240", "3242", "3244", "3246",
                    "3080",
                ],
                &["3080", "3081"],
                &[
                    "3050", "3052", "3054", "3056", "3058", "3060", "3062", "3064", "3066", "3068",
                    "3070", "3072", "3074", "3076", "3078", "3080",
                ],
            ],
        );
        assert_eq!(components, Some(vec![12, 11, 8]));
    }
}
