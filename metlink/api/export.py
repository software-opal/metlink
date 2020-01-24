from ..utils import json_load
from ..utils import pretty_json_dump as json_dump
from .db import Service, ServiceRouteMap, Stop, create_db, db_session

dump_files = [
    ("stops.json", Stop),
    ("services.json", Service),
    ("service_maps.json", ServiceRouteMap),
]


def dump_data(db, table, file):
    records = db.query(table).all()
    names = [column.name for column in table.__mapper__.columns]
    output = []
    for row in records:
        output.append({name: getattr(row, name) for name in names})
    json_dump({"data": output}, file, sort_keys=True, indent=2)


def load_data(db, table, file):
    data = json_load(file)["data"]

    db.query(table).delete()
    db.add_all([table(**row) for row in data])


def export_all_types():
    with db_session() as db:
        for file, table in dump_files:
            with open(file, "w") as f:
                dump_data(db, table, f)


def import_all_types():
    with db_session() as db:
        for file, table in dump_files:
            try:
                with open(file, "r") as f:
                    load_data(db, table, f)
            except IOError:
                pass


def main():
    create_db()
    export_all_types()


if __name__ == "__main__":
    main()
