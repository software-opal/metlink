import decimal
import typing as typ

from ..session import get_session
from ..utils import LAT_LON_EXPONENT, decimal_parse, json_dumps, save_response
from . import API_V1_BASE
from .db import Service, ServiceRouteMap, check_or_make, create_db, db_session
from .services import import_services

Coordinate = typ.Tuple[decimal.Decimal, decimal.Decimal]


def distance(lat_lon, coord):
    return decimal.Decimal(
        (lat_lon[0] - coord[0]) ** 2 + (lat_lon[0] - coord[0]) ** 2
    ).sqrt()


def sort_stops_by_closest(
    lat_lon: Coordinate, stop_positions: typ.Dict[Coordinate, str]
) -> typ.List[typ.Tuple[Coordinate, str, decimal.Decimal]]:
    with decimal.localcontext() as ctx:
        ctx.prec = 12  # Work with high precision math for the distance
        pos_stops_distances = [
            (coord, stop, distance(lat_lon, coord),)
            for coord, stop in stop_positions.items()
        ]
    # Sorted will put the lowest distance first
    return sorted(pos_stops_distances, key=lambda item: item[2])


def find_route_stops(
    route_positions: typ.List[Coordinate], stop_positions: typ.Dict[Coordinate, str]
) -> typ.Iterator[typ.List[str]]:
    start_stops = sort_stops_by_closest(route_positions[0], stop_positions)
    end_stops = sort_stops_by_closest(route_positions[-1], stop_positions)
    # Stop presenting options when the stop is more than ~10m from the coordinate given
    epsilon = 7e-5
    for (_, start_stop, start_epsilon) in start_stops:
        if start_epsilon > epsilon:
            break
        for (_, end_stop, end_epsilon) in end_stops:
            if end_epsilon > epsilon:
                break
            yield [start_stop, end_stop]


def map_coords_to_location_dicts(
    positions: typ.List[Coordinate],
) -> typ.List[typ.Dict[str, typ.Any]]:
    return


def map_coord_to_location_dict(
    position: Coordinate, stop: typ.Optional[str] = None
) -> typ.Dict[str, typ.Any]:
    re = {"lat": position[1], "lon": position[0]}
    if stop is not None:
        re["stop"] = stop
    return re


def find_route_stop_and_map_locations(
    route_positions: typ.List[Coordinate], stop_positions: typ.Dict[Coordinate, str]
) -> typ.Iterator[typ.Tuple[typ.List[typ.Dict[str, typ.Any]], typ.List[str]]]:
    for route_stops in find_route_stops(route_positions, stop_positions):
        yield (
            (
                [
                    {
                        "lat": route_positions[0][0],
                        "lon": route_positions[0][1],
                        "stop": route_stops[0],
                    }
                ]
                + [{"lat": lat, "lon": lon} for lat, lon in route_positions[1:-1]]
                + [
                    {
                        "lat": route_positions[-1][0],
                        "lon": route_positions[-1][1],
                        "stop": route_stops[-1],
                    }
                ]
            ),
            route_stops,
        )


# The caps come from the JSON, don't change them <3


def load_service_maps(
    service_code,
    *,
    Code,
    LastModified,
    Link,
    Mode,
    Name,
    RouteMaps,
    StopLocations,
    TrimmedCode,
    **kwargs,
):
    if Code.upper() != service_code.upper():
        print(f"Warning(Svc {service_code}): Inconsistent service codes")
        return []
    del LastModified, Link, Mode, Name, TrimmedCode, Code
    if kwargs:
        print(
            f"Warning(Svc {service_code}):"
            f" extra service arguments recieved: {list(kwargs)}"
        )
    stops = []
    lat_lon_to_stop_id = {}
    stop_id_to_lat_lon = {}
    for stop in StopLocations:
        stop_id = stop["Sms"]
        lat, lon = list(map(decimal_parse, stop["LatLng"].split(",")[:2]))
        lat_lon_to_stop_id[(lat, lon)] = stop_id[:4]
        stop_id_to_lat_lon.setdefault(stop_id, []).append((lat, lon))

    svc_routes = []
    for i, route in enumerate(RouteMaps):
        route_positions = [
            (lat, lon)
            for lat, lon in [
                list(map(decimal_parse, entry.split(",")[:2]))
                for entry in route["Path"]
            ]
        ]
        stops_and_locs = find_route_stop_and_map_locations(
            route_positions, lat_lon_to_stop_id
        )
        if stops_and_locs:
            for route_path, route_stops in stops_and_locs:
                svc_routes.append(
                    ServiceRouteMap(
                        code=service_code,
                        stops_str=json_dumps(route_stops),
                        route_str=json_dumps(route_path),
                    )
                )
        else:
            print(
                f"Unable to find a route with any stops for {service_code}/{i}",
                route_positions,
                lat_lon_to_stop_id,
            )
            raise ValueError("AAAAA")
    return svc_routes


def import_service_maps():
    check_or_make(Service, import_services)

    with db_session() as db:
        service_codes = [s.code for s in db.query(Service).all()]
    with get_session() as req:
        with db_session() as db:
            for svc_code in service_codes:
                print(f"Service: {svc_code}")
                url = f"{API_V1_BASE}/ServiceMap/{svc_code.upper()}"
                with req.get(url) as resp:
                    save_response(resp)
                    resp.raise_for_status()
                    data = resp.json()
                db.query(ServiceRouteMap).filter(
                    ServiceRouteMap.code == svc_code
                ).delete()
                new_route_maps = load_service_maps(svc_code, **data)
                for map in new_route_maps:
                    db.add(map)


def main():
    create_db()

    import_service_maps()


if __name__ == "__main__":
    main()
