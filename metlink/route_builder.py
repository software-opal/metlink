import functools

from .utils import BASE, decimal_parse, json_load

responses = BASE / "responses"

sl = json_load(
    (responses / "https___www.metlink.org.nz_api_v1_ServiceList_.json").open("r")
)


def distance_sq(a, b):
    return (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2


def find_closest_stop(position, stops):
    closest_stop = None
    closest_stop_dist_sq = distance_sq((0, 0), position)
    for stop, stop_pos in stops.items():
        stop_dist_sq = distance_sq(position, stop_pos)
        if closest_stop_dist_sq > stop_dist_sq:
            closest_stop_dist_sq = stop_dist_sq
            closest_stop = stop
    return closest_stop


def find_possible_routes(route_stops, rps_start_stop_map):
    if not route_stops:
        return []
    start = route_stops[0]
    if start not in rps_start_stop_map:
        return []
    poss_routes = []
    for end, route_ids in rps_start_stop_map[start].items():
        if end in route_stops[1:]:
            end_idx = route_stops.index(end, 1)
            if end_idx == (len(route_stops) - 1):
                poss_routes += [[rid] for rid in route_ids]
            else:
                subroutes = find_possible_routes(
                    route_stops[end_idx:], rps_start_stop_map
                )
                poss_routes += [
                    [rid] + subroute for subroute in subroutes for rid in route_ids
                ]
    return poss_routes


def build_route_from_ids(route_ids, route_path_segments):
    poss_route_coords = []
    for rps_id in route_ids:
        route_coords = route_path_segments[rps_id]
        if poss_route_coords and poss_route_coords[-1] == route_coords[0]:
            poss_route_coords += route_coords[1:]
        else:
            poss_route_coords += route_coords
    return poss_route_coords


def load_service(code):
    sm = json_load(
        (responses / f"https___www.metlink.org.nz_api_v1_ServiceMap_{code}.json").open(
            "r"
        )
    )
    stops = {
        s["Sms"]: tuple(map(decimal_parse, s["LatLng"].split(",")))[:2]
        for s in sm["StopLocations"]
    }
    route_path_segments = [
        [tuple(map(decimal_parse, ps.split(",")))[:2] for ps in r["Path"]]
        for r in sm["RouteMaps"]
    ]
    route_path_segment_start_end_stops = [
        (find_closest_stop(rps[0], stops), find_closest_stop(rps[-1], stops))
        for rps in route_path_segments
    ]
    rps_start_stop_map = {}
    for i, (start_, end_) in enumerate(route_path_segment_start_end_stops):
        # We need to handle the 5-letter stations gracefully; so we'll point
        # both of them at the same rps id.
        for start in set((start_, start_[:4])):
            for end in set((end_, end_[:4])):
                rps_start_stop_map.setdefault(start, {}).setdefault(end, []).append(i)

    routes = set()
    for file in (BASE / f"data/service-{code}/timetables").iterdir():
        if not file.is_file() and file.suffix != ".json":
            continue
        ttbl = json_load(file.open("r"))
        for ttbl_svc in ttbl["timetables"]:
            routes.add(tuple(ttbl_svc["stops"]))
    for route in routes:
        possible_route_ids = find_possible_routes(route, rps_start_stop_map)
        possible_routes_coords = [
            build_route_from_ids(poss_route, route_path_segments)
            for poss_route in possible_route_ids
        ]
        try:
            ttbld_route_coords = [stops[stop] for stop in route]
        except KeyError:
            print(f"Stops not found for {code}/{route}")
            continue
        if len(possible_routes_coords) == 0:
            print(f"Unable to find route for {code}/{route}")
            print(rps_start_stop_map)
            raise ValueError("")
        elif len(possible_routes_coords) == 1:
            best_route_idx = 0
        else:
            print(
                f"Routing issue. Trying to resolve the bestest route from {len(possible_routes_coords)}"
            )
            best_route_idx = min(
                (
                    sum(
                        min(
                            distance_sq(stop_coord, coord)
                            for coord in poss_route_coords
                        )
                        for stop_coord in ttbld_route_coords
                    ),
                    i,
                )
                for i, poss_route_coords in enumerate(possible_routes_coords)
            )[1]


for svc in sl:
    code = svc["Code"]
    print(f"Code: {code}")
    # if code >= '290':
    load_service(code)
    # break
