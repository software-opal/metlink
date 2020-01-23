from .db import create_db
from .export import export_all_types
from .service_maps import import_service_maps
from .services import import_services
from .stops import import_stops


def main():
    create_db()
    import_services()
    import_stops()
    import_service_maps()
    export_all_types()


if __name__ == "__main__":
    main()
