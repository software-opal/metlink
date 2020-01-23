from .api.db import create_db
from .api.export import export_all_types
from .api.service_maps import import_service_maps
from .api.services import import_services
from .api.stops import import_stops
from .geo.route_builder import convert_service_maps
from .timetables.download import load as load_timetables


def main():
    create_db()
    import_services()
    import_stops()
    import_service_maps()
    export_all_types()
    convert_service_maps()
    load_timetables()


if __name__ == "__main__":
    main()
