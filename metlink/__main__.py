from .api.db import create_db
from .api.export import export_all_types
from .api.service_maps import import_service_maps
from .api.services import import_services
from .api.stops import import_stops
from .geo.route_builder import convert_service_maps
from .timetables.aggregate import generate_aggregates
from .timetables.download import load as load_timetables

import time


def main():
    create_db()
    print("import_services")
    start = time.monotonic()
    import_services()
    print("end import_services: %02d:%02d" % divmod(time.monotonic() - start, 60))
    print("import_stops")
    start = time.monotonic()
    import_stops()
    print("end import_stops: %02d:%02d" % divmod(time.monotonic() - start, 60))
    print("import_service_maps")
    start = time.monotonic()
    import_service_maps()
    print("end import_service_maps: %02d:%02d" % divmod(time.monotonic() - start, 60))
    print("export_all_types")
    start = time.monotonic()
    export_all_types()
    print("end export_all_types: %02d:%02d" % divmod(time.monotonic() - start, 60))
    print("convert_service_maps")
    start = time.monotonic()
    convert_service_maps()
    print("end convert_service_maps: %02d:%02d" % divmod(time.monotonic() - start, 60))
    # try:
    #     print("generate_aggregates")
    #     start = time.monotonic()
    #     generate_aggregates()
    #     print(
    #         "end generate_aggregates: %02d:%02d" % divmod(time.monotonic() - start, 60)
    #     )
    # except Exception:
    #     print("end generate_aggregates: FAILED")
    # print("load_timetables")
    # start = time.monotonic()
    # load_timetables()
    # print("end load_timetables: %02d:%02d" % divmod(time.monotonic() - start, 60))
    print("generate_aggregates")
    start = time.monotonic()
    generate_aggregates()
    print("end generate_aggregates: %02d:%02d" % divmod(time.monotonic() - start, 60))


if __name__ == "__main__":
    main()
