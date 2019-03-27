from .session import get_session
from . import API_V1_BASE
from .db import Session, create_db, Stops
import decimal
import datetime as dt


# The caps come from the JSON, don't change them <3
def load_stop(*, Name, Sms, Farezone, Lat, Long, LastModified, **kwargs):
    if kwargs:
        print(f"Warning(Stop {Sms}): extra stop arguments recieved: {list(kwargs)}")
    try:
        with decimal.localcontext() as ctx:
            # MetLink returns lat/long with 7 decimals, which is ~11mm at the equator.
            ctx.precision = 7
            lat = decimal.Decimal(Lat)
            long = decimal.Decimal(Long)
    except decimal.DecimalException as e:
        print(f"Warning(Stop {Sms}): Failed to convert Lat/Long({Lat!r}/{Long!r}): {e}")
        return None
    try:
        # 2019-03-27T00:00:09+13:00
        last_modified = dt.datetime.strptime(LastModified, "%Y-%m-%dT%H:%M:%S%z")
    except ValueError as e:
        print(
            f"Warning(Stop {Sms}): Failed to convert LastModified({LastModified!r}): {e}"
        )
    del Lat, Long, LastModified  # Save me some pain debugging

    return Stops(
        name=Name,
        sms=Sms,
        fare_zone=Farezone,
        lat=lat,
        long=long,
        last_modified=last_modified,
    )


def main():
    create_db()
    with get_session().get(f"{API_V1_BASE}/StopList/") as resp:
        resp.raise_for_status()
        data = resp.json()
    session = Session()
    session.add_all(list(filter(None, (load_stop(**stop) for stop in data["Stops"]))))
    session.commit()


if __name__ == "__main__":
    main()
