import decimal
import typing as typ

from ..session import get_session
from ..utils import LAT_LON_EXPONENT, decimal_parse, json_dumps, save_response
from . import API_V1_BASE
from .db import Service, ServiceRouteMap, check_or_make, create_db, db_session
from .services import import_services

Coordinate = typ.Tuple[decimal.Decimal, decimal.Decimal]


def find_stop_within_epsilon(lat_lon: Coordinate, stop_positions, epsilon):
    if epsilon == 0:
        return stop_positions.get(lat_lon, None)
    lat, lon = lat_lon
    for (test_lat, test_lon), stop in stop_positions.items():
        if abs(test_lat - lat) <= epsilon and abs(test_lon - lon) <= epsilon:
            return stop


def find_route_stops(
    route_positions: typ.List[Coordinate], stop_positions: typ.Dict[Coordinate, str]
) -> typ.Tuple[typ.List[str], typ.Dict[str, typ.Any]]:
    # This looks for stops at 1m-10m away.
    accuracies = range(1, 10)
    for accuracy in accuracies:
        epsilon = LAT_LON_EXPONENT * accuracy
        route_path = []
        route_stops = []

        for lat, lon in route_positions:
            stop_id = find_stop_within_epsilon((lat, lon), stop_positions, epsilon)
            if stop_id is not None:
                if len(route_stops) == 0 or route_stops[-1] != stop_id:
                    route_stops.append(stop_id)
                route_path.append({"lat": lat, "lon": lon, "stop": stop_id})
            else:
                route_path.append({"lat": lat, "lon": lon})

        if len(route_stops) >= 2:
            return route_path, route_stops
    return None, None


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
        route_path, route_stops = find_route_stops(route_positions, lat_lon_to_stop_id)
        if route_stops:
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
    return svc_routes


def import_service_maps():
    check_or_make(Service, import_services)

    with db_session() as db:
        service_codes = [s.code for s in db.query(Service).all()]
    with get_session() as req:
        for svc_code in service_codes:
            print(f"Service: {svc_code}")
            url = f"{API_V1_BASE}/ServiceMap/{svc_code.upper()}"
            with req.get(url) as resp:
                save_response(resp)
                resp.raise_for_status()
                data = resp.json()
            with db_session() as db:
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
