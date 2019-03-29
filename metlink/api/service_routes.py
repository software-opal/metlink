from . import API_V1_BASE, API_TIMETABLE_BASE
from .db import Service, ServiceRoute, create_db, db_session
from .session import get_session

import datetime as dt
import json

next_dow = {
    d.

}

# The caps come from the JSON, don't change them <3
def load_service_routes(service, direction, *, points, lines, **kwargs):
    if kwargs:
        print(
            f"Warning(Svc {service.code}/{direction}):"
            f" extra service arguments recieved: {list(kwargs)}"
        )

    return ServiceRoute(
        code=service.code,
        direction=direction,
        points_str=json.dumps(points, sort_keys=True, separators=(",", ":")),
        lines_str=json.dumps(lines, sort_keys=True, separators=(",", ":")),
    )

def download_service_route(service, direction):
    url = (f"{API_TIMETABLE_BASE}/{service.mode.lower()}/"
    f"{service.code.upper()}/{direction}/mapdatajson")

        with req.get() as resp:
            resp.raise_for_status()
            data = resp.json()


def import_service_routes():
    with db_session() as session:
        service_count = session.query(Service).count()
    if service_count == 0:
        from .services import import_services

        import_services()

    with db_session() as db, get_session() as req:
        services = session.query(Service).all()
        for service in services:
            for direction in ["inbound", "outbound"]:

                    route = load_service_routes(service, direction, **data)
                    if route:
                        db.add(route)
                data = import_service_route()


def main():
    create_db()
    import_service_routes()


if __name__ == "__main__":
    main()
