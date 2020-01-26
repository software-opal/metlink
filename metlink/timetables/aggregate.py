from ..api.db import Service, check_or_make, create_db, db_session
from ..api.services import import_services
from ..utils import BASE, json_load, pretty_json_dump

DATA = BASE / "data/"


def aggregate_service(data_dir, svc_code):
    timetable_json = data_dir / f"service-{svc_code}/timetables.json"
    timetable_folder = data_dir / f"service-{svc_code}/timetables/"
    timetable_folder.mkdir(parents=True, exist_ok=True)

    timetables = []
    for timetable_file in timetable_folder.iterdir():
        if timetable_file.suffix == ".json":
            try:
                with timetable_file.open("r") as f:
                    timetables.append(json_load(f))
            except (IOError, ValueError):
                pass
    timetables.sort(key=lambda t: (t["day"], t["direction"]))
    with timetable_json.open("w") as f:
        pretty_json_dump(timetables, f)


def generate_aggregates():
    check_or_make(Service, import_services)
    services = []
    with db_session() as db:
        for svc in db.query(Service).all():
            services.append(svc.code)
    for service in services:
        aggregate_service(DATA, service)


def main():
    create_db()
    load()


if __name__ == "__main__":
    main()
