import pathlib

from ..api.db import (
    Service,
    ServiceRouteMap,
    Stop,
    check_or_make,
    create_db,
    db_session,
)
from ..api.service_maps import import_service_maps
from ..api.services import import_services
from ..api.stops import import_stops
from ..utils import decimal_parse, pretty_json_dump

ROOT = (pathlib.Path(__file__).parent / "../..").resolve()


def geojson_route(service, route_name, route):
    return {
        "type": "Feature",
        "properties": {
            "stroke-opacity": 0.3,
            "name": service.name,
            "code": service.code,
            "mode": service.mode,
            "route_id": route_name,
        },
        "geometry": {
            "type": "LineString",
            "coordinates": [
                [path_item["lon"], path_item["lat"]] for path_item in route.route
            ],
        },
    }


def geojson_stop(stop):
    return {
        "type": "Feature",
        "properties": {
            "name": stop.name,
            "sms": stop.sms,
            "farezone": stop.fare_zone,
            "marker-size": "small",
        },
        "geometry": {
            "type": "Point",
            "coordinates": [decimal_parse(stop.long), decimal_parse(stop.lat)],
        },
    }


def output_service(data_dir, service, service_maps, all_stops):
    service_folder = data_dir / f"service-{service.code}/"
    service_folder.mkdir(parents=True, exist_ok=True)

    for file in service_folder.iterdir():
        # We remove all the geojson files in the service's folder so we are
        #  sure we have a clean slate
        if file.is_file() and file.suffix == ".geojson":
            file.unlink()

    data = {
        "name": service.name,
        "code": service.code,
        "mode": service.mode,
        "schools": service.schools,
        "last_modified": service.last_modified,
        "stops": [],
        "routes": [],
    }
    serviced_stops = set()
    service_route_start_end_stops = {}
    service_route_features = []

    for route in service_maps:
        stops = route.stops
        start, end = stops[0], stops[-1]
        base_route_name = f"{start}-{end}"
        start_stop_routes = service_route_start_end_stops.setdefault((start, end), [])
        start_stop_routes.append(route)
        serviced_stops.update(stops)
        if len(start_stop_routes) == 1:
            route_name = base_route_name
        else:
            route_name = f"{base_route_name}-{len(start_stop_routes)}"
        data["routes"].append(
            {
                "id": route_name,
                "start_id": start,
                "end_id": end,
                "stops": stops,
                "route": route.route,
            }
        )
        route_feature = geojson_route(service, route_name, route)
        service_route_features.append(route_feature)
        with (service_folder / f"{route_name}.geojson").open("w") as f:
            pretty_json_dump(
                {
                    "type": "FeatureCollection",
                    "features": [route_feature]
                    + [geojson_stop(all_stops[stop_id]) for stop_id in stops],
                },
                f,
            )
    data["stops"] = [
        {
            "name": all_stops[stop_id].name,
            "sms": all_stops[stop_id].sms,
            "farezone": all_stops[stop_id].fare_zone,
            "lat": decimal_parse(all_stops[stop_id].lat),
            "lon": decimal_parse(all_stops[stop_id].long),
        }
        for stop_id in serviced_stops
    ]

    with (service_folder / f"service.geojson").open("w") as f:
        pretty_json_dump(
            {
                "type": "FeatureCollection",
                "features": service_route_features
                + [
                    geojson_stop(all_stops[stop_id])
                    for stop_id in sorted(serviced_stops)
                ],
            },
            f,
        )
    with (service_folder / "service.json").open("w") as f:
        pretty_json_dump(data, f)


def convert_service_maps():
    check_or_make(Service, import_services)
    check_or_make(Stop, import_stops)
    check_or_make(ServiceRouteMap, import_service_maps)

    with db_session() as db:
        all_services = {svc.code: svc for svc in db.query(Service).all()}
        all_stops = {stop.sms: stop for stop in db.query(Stop).all()}
        all_service_maps = {}
        for sr in db.query(ServiceRouteMap).all():
            all_service_maps.setdefault(sr.code, []).append(sr)
    with (ROOT / "data" / "stops.json").open("w") as f:
        pretty_json_dump(all_stops, f)
    for service in all_services.values():
        output_service(
            ROOT / "data/", service, all_service_maps[service.code], all_stops
        )


def main():
    create_db()

    convert_service_maps()


if __name__ == "__main__":
    main()
