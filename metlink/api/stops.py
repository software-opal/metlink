import datetime as dt
import decimal

from . import API_V1_BASE
from .db import Stop, create_db, db_session
from .session import get_session


# The caps come from the JSON, don't change them <3
def load_stop(*, Name, Sms, Farezone, Lat, Long, LastModified, **kwargs):
    if kwargs:
        print(f"Warning(Stop {Sms}): extra stop arguments recieved: {list(kwargs)}")
    try:
        with decimal.localcontext() as ctx:
            # MetLink returns lat/long with 7 decimals, which is ~11mm at the equator.
            ctx.prec = 7
            lat = decimal.Decimal(Lat)
            long = decimal.Decimal(Long)
    except decimal.DecimalException as e:
        print(f"Warning(Stop {Sms}): Failed to convert Lat/Long({Lat!r}/{Long!r}): {e}")
        return None
    del Lat, Long  # Save me some pain debugging

    return Stop(
        name=Name,
        sms=Sms,
        fare_zone=Farezone,
        lat=str(lat),
        long=str(long),
        last_modified=LastModified,
    )


def import_stops():
    with get_session().get(f"{API_V1_BASE}/StopList/") as resp:
        resp.raise_for_status()
        data = resp.json()
    with db_session() as session:
        session.add_all(
            list(filter(None, (load_stop(**stop) for stop in data["Stops"])))
        )


def main():
    create_db()
    import_stops()


if __name__ == "__main__":
    main()
