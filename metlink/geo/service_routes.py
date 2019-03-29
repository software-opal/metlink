import datetime as dt
import decimal
import geojson

from ..api.db import ServiceRoute, create_db, db_session


def convert_route_to_geojson(route: ServiceRoute):
    if len(route.lines) != 1:
        print(
            f"Route {route.code}/{route.direction} has weird number of lines: {len(route.lines)}"
        )
    line_nodes = []

    with decimal.localcontext() as ctx:
        ctx.prec = 7
        for line in route.lines:
            for node in line["path"]:
                lat, lon, _ = node.split(",")
                line_nodes.append((float(lon), float(lat)))
    return geojson.LineString(line_nodes)


def main():
    create_db()
    with db_session() as db:
        routes = db.query(ServiceRoute).count()
    if routes == 0:
        from ..api.service_routes import import_service_routes

        import_service_routes()
    with db_session() as db:
        routes = db.query(ServiceRoute).all()
    lines = []
    for route in routes:
        line = convert_route_to_geojson(route)
        lines.append(geojson.Feature(f'{route.code}/{route.direction}', line))
    print(routes)
    with open("lines.geojson", 'w') as f:
        geojson.dump(geojson.FeatureCollection(lines), f, indent=2)


if __name__ == "__main__":
    main()
