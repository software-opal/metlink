import datetime as dt
import json

from . import API_TIMETABLE_BASE, API_V1_BASE
from .db import Service, ServiceRoute, create_db, db_session
from .session import get_session

next_dow = {
    d.weekday(): d
    for d in (dt.date.today() + dt.timedelta(days=i) for i in range(1, 8))
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


def download_service_route(req, service, direction):
    url = (
        f"{API_TIMETABLE_BASE}/{service.mode.lower()}/"
        f"{service.code.upper()}/{direction}/mapdatajson"
    )
    # Today, Fri, Mon, Wedr, Sat, Tue, Thu, Sun
    for weekday in [None, 4, 0, 2, 5, 1, 3, 6]:
        params = {}
        if weekday is not None:
            params["date"] = next_dow[weekday]
        with req.get(
            url, params={"date": None if weekday is None else next_dow[weekday]}
        ) as resp:
            if resp.status_code == 404:
                # The bus does not run today, so we can't see it's route
                # Why? *shrug*
                continue
            resp.raise_for_status()
            return resp.json()
    print(
        f"Warning: Service({service.code}/{direction}) "
        "does not appear to have any routes for the next 7 days"
    )
    return None


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
                print(f"Service: {service.code}/{direction}")
                data = download_service_route(req, service, direction)
                if data:
                    route = load_service_routes(service, direction, **data)
                    if route:
                        db.add(route)


def main():
    create_db()
    import_service_routes()


if __name__ == "__main__":
    main()
