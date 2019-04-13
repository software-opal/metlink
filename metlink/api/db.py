import contextlib
import pathlib

from sqlalchemy import Column, ForeignKey, Integer, String, create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

from ..utils import decimal_parse, json_dumps, json_loads

ROOT = (pathlib.Path(__file__).parent / "../..").resolve()
DB = ROOT / "db.sqlite3"
Base = declarative_base()
Session = sessionmaker()


class Stop(Base):
    __tablename__ = "stops"
    name = Column(String)
    sms = Column(String, primary_key=True)
    fare_zone = Column(String)
    lat = Column(String)  # Numeric(10, 7)
    long = Column(String)  # Numeric(10, 7)
    last_modified = Column(String)

    @property
    def position(self):
        return (decimal_parse(self.lat), decimal_parse(self.long))

    @position.setter
    def position(self, value):
        raw_lat, raw_long = value
        (self.lat, self.long) = (
            str(decimal_parse(raw_lat)),
            str(decimal_parse(raw_long)),
        )


class Service(Base):
    __tablename__ = "services"
    name = Column(String)
    code = Column(String, primary_key=True)
    mode = Column(String)
    link = Column(String)
    last_modified = Column(String)
    schools_str = Column(String)

    @property
    def schools(self):
        if self.schools_str:
            return list(filter(None, map(str.strip, self.schools_str.split(","))))
        else:
            return []

    @schools.setter
    def schools(self, value):
        if not value:
            self.schools_str = None
        for v in value:
            assert "," not in v
        self.schools_str = ", ".join(v.strip() for v in value)


class ServiceRouteMap(Base):
    __tablename__ = "service_route_map"

    id = Column(Integer, primary_key=True)
    code = Column(String, ForeignKey("services.code"))
    stops_str = Column(String)
    route_str = Column(String)

    @property
    def stops(self):
        route_stops = []
        for stop_id in json_loads(self.stops_str):
            if len(route_stops) == 0 or route_stops[-1] != stop_id:
                route_stops.append(stop_id)
        return route_stops

    @stops.setter
    def stops(self, data):
        self.stops_str = json_dumps(data)

    @property
    def route(self):
        return json_loads(self.route_str)

    @route.setter
    def route(self, data):
        self.route_str = json_dumps(data)


class ServiceTimetable(Base):
    __tablename__ = "service_timetable"

    service = Column(String, ForeignKey("services.code"), primary_key=True)
    date = Column(String, primary_key=True)


@contextlib.contextmanager
def db_session():
    session = Session()
    try:
        yield session
    except Exception as e:
        session.rollback()
        raise e
    else:
        session.commit()


def create_db():
    engine = get_engine()
    Session.configure(bind=engine)
    Base.metadata.create_all(engine)


def get_connection():
    return get_engine().connect()


def get_engine():
    return create_engine(f"sqlite:///{DB}")


def check_or_make(model, maker):
    with db_session() as session:
        count = session.query(model).count()
    if count == 0:
        maker()
