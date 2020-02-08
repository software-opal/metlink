import datetime as dt
from multiprocessing import Pool
import time

import pytz

from ..api.db import Service, check_or_make, create_db, db_session
from ..api.services import import_services
from ..session import get_session
from ..utils import BASE, parse_html, pretty_json_dumps

WELLINGTON_TZ = pytz.timezone("Pacific/Auckland")
DATA = BASE / "data/"
TIMETABLED_DAYS = [
    dt.date.today() + dt.timedelta(days=t)
    # Today to 28ish days in the future
    # The timetables appear to go about 40ish days in the future
    for t in range(12, 35)
]


def parse_timetable(page_content):
    doc = parse_html(page_content)
    stops = []
    times = []
    for row in doc.find(id="timetableData").tbody.find_all("tr"):
        stops.append(row["data-sms"])
        stop_times = [
            dt.datetime.fromtimestamp(int(td.span["data-time"]), WELLINGTON_TZ)
            if td.span
            else None
            for td in row.find_all("td")
        ]
        times.append(stop_times)
    timetable = []
    for run_times in zip(*times):
        run_stop_times = []
        for stop, time in zip(stops, run_times):
            if time is None:
                continue
            run_stop_times.append((stop, time))
        timetable.append(run_stop_times)
    return timetable


def output_timetable(data_dir, svc_code, direction, date, timetable):
    timetable_folder = data_dir / f"service-{svc_code}/timetables/"
    timetable_folder.mkdir(parents=True, exist_ok=True)

    timetable_remapped = [tuple(zip(*tbl)) for tbl in timetable]

    with (timetable_folder / f"{date:%Y-%m-%d}-{direction}.json").open("w") as f:
        pass
        f.write(
            pretty_json_dumps(
                {
                    "service": svc_code,
                    "direction": direction,
                    "day": f"{date:%Y-%m-%d}",
                    "timetables": [
                        {"stops": list(stops), "times": [t.isoformat() for t in times]}
                        for stops, times in timetable_remapped
                    ],
                }
            )
        )


def parse_and_save_timetable(url, code, date):

    print(f"Timetable for {code} on {date}")
    start = time.monotonic()

    with get_session() as sess:
        for direction in ["inbound", "outbound"]:
            with sess.get(f"{url}/{direction}?date={date:%Y-%m-%d}") as resp:
                if (
                    "service does not have a timetable" not in resp.text
                    and "Service not found" not in resp.text
                ):
                    timetable = parse_timetable(resp.text)
                    output_timetable(DATA, code, direction, date, timetable)
    print(
        f"Timetable for {code} on {date}:"
        + ("%02d:%02d" % divmod(time.monotonic() - start, 60))
    )


def load():
    check_or_make(Service, import_services)
    base_urls = []
    with db_session() as db:
        for svc in db.query(Service).all():
            base_urls.append((svc.code, f"https://www.metlink.org.nz{svc.link}"))
    with Pool(32) as p:
        p.starmap(parse_and_save_timetable, (
            (url, code, date)
            for date in TIMETABLED_DAYS
            for code, url in base_urls
        ))


def main():
    create_db()
    load()


if __name__ == "__main__":
    main()
