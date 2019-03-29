import contextlib
import decimal
import pathlib
import json

from sqlalchemy import Column, ForeignKey, String, create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

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
        with decimal.localcontext() as ctx:
            ctx.prec = 7
            # MetLink returns lat/long with 7 decimals, which is ~11mm at the equator.
            return (decimal.Decimal(self.lat), decimal.Decimal(self.long))

    @position.setter
    def position(self, value):
        raw_lat, raw_long = value
        with decimal.localcontext() as ctx:
            ctx.prec = 7
            (self.lat, self.long) = (
                str(decimal.Decimal(raw_lat)),
                str(decimal.Decimal(raw_long)),
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
        return [school.trim() for school in self.school_str.split(",") if school.trim()]

    @schools.setter
    def schools(self, value):
        for v in value:
            assert "," not in v
        self.schools_str = ", ".join(v.strip() for v in value)


class ServiceRoute(Base):
    __tablename__ = "service_routes"

    code = Column(String, ForeignKey("services.name"), primary_key=True)
    direction = Column(String, primary_key=True)
    points_str = Column(String)
    lines_str = Column(String)

    @property
    def points(self):
        return json.loads(self.points_str)

    @points.setter
    def points(self, data):
        self.points_str = json.dumps(data, sort_keys=True, separators=(",", ":"))

    @property
    def lines(self):
        return json.loads(self.lines_str)

    @lines.setter
    def lines(self, data):
        self.lines_str = json.dumps(data, sort_keys=True, separators=(",", ":"))


@contextlib.contextmanager
def db_session():
    session = Session()
    try:
        yield session
    except:
        session.rollback()
        raise
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
