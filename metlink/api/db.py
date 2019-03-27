import pathlib

from sqlalchemy import Column, String, create_engine, Numeric
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
import decimal

ROOT = (pathlib.Path(__file__).parent / "../..").resolve()
DB = ROOT / "db.sqlite3"

Base = declarative_base()
Session = sessionmaker()


class Stops(Base):
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
            ctx.precision = 7
            # MetLink returns lat/long with 7 decimals, which is ~11mm at the equator.
            return (decimal.Decimal(self.lat), decimal.Decimal(self.long))

    @position.setter
    def position(self, value):
        raw_lat, raw_long = value
        with decimal.localcontext() as ctx:
            ctx.precision = 7
            (self.lat, self.long) = (
                str(decimal.Decimal(raw_lat)),
                str(decimal.Decimal(raw_long)),
            )


class Services(Base):
    __tablename__ = "services"
    name = Column(String)
    code = Column(String, primary_key=True)
    mode = Column(String)
    last_modified = Column(String)


def create_db():
    engine = get_engine()
    Session.configure(bind=engine)
    Base.metadata.create_all(engine)


def get_connection():
    return get_engine().connect()


def get_engine():
    return create_engine(f"sqlite:///{DB}")
