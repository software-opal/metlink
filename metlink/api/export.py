import json

from .db import Service, ServiceRoute, Stop, create_db, db_session


def dump_data(db, table, file):
    records = db.query(table).all()
    names = [column.name for column in table.__mapper__.columns]
    output = []
    for row in records:
        output.append({name: getattr(row, name) for name in names})
    json.dump({"data": output}, file, sort_keys=True, indent=2)


def main():
    create_db()
    with db_session() as db:
        for file, table in [
            ("stops.json", Stop),
            ("services.json", Service),
            ("service_routes.json", ServiceRoute),
        ]:
            with open(file, "w") as f:
                dump_data(db, table, f)


if __name__ == "__main__":
    main()
