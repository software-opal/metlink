import datetime as dt
import pprint

import pytz
import pathlib

from ..utils import decimal_parse, pretty_json_dumps
from ..api.db import Service, create_db, db_session, check_or_make
from ..session import get_session
from ..api.services import import_services
from ..utils import parse_html

WELLINGTON_TZ = pytz.timezone("Pacific/Auckland")
ROOT = pathlib.Path(__file__).parent / "../.."
DATA = ROOT / "data/"
TIMETABLED_DAYS = [
    dt.date.today() + dt.timedelta(days=t)
    # Today to 14ish days in the future
    for t in range(0, 14)
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
        start = None
        run_stop_times = []
        for stop, time in zip(stops, run_times):
            if time is None:
                continue
            run_stop_times.append((stop, time))
        timetable.append(run_stop_times)
    return timetable


def determine_run_times(timetables):
    route_start_end = []
    for route_times in timetables:
        times = [time for _, time in route_times]
        route_start_end.append((min(times), max(times)))
    route_start_end = sorted(route_start_end)
    ranges = []
    r_start, r_end = route_start_end[0]
    for start, end in route_start_end:
        if r_start <= start <= r_end or r_start <= end <= r_end:
            r_start = min(r_start, start)
            r_end = max(r_end, end)
        else:
            ranges.append((r_start, r_end))
            r_start, r_end = start, end
    ranges.append((r_start, r_end))
    ranges = sorted(ranges)
    for (r_start, r_end) in ranges:
        assert r_start < r_end, f"Incorrect range ({r_start}, {r_end})"
    for r_start, r_end in ranges:
        for t_start, t_end in ranges:
            if t_start == r_start and t_end == t_end:
                pass  # Same, skip
            elif r_start <= t_start <= r_end or r_start <= t_end <= r_end:
                raise ValueError(
                    f"Time range ({r_start}, {r_end}) overlaps with ({t_start}, {t_end})"
                )
    return ranges


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


def load():
    check_or_make(Service, import_services)
    with db_session() as db:
        service_codes = [s.code for s in db.query(Service).all()]
    base_urls = []
    with db_session() as db:
        for svc in db.query(Service).all():
            base_urls.append((svc.code, f"https://www.metlink.org.nz{svc.link}"))
    with get_session() as sess:
        for date in TIMETABLED_DAYS:
            for code, url in base_urls:
                print(code, date)
                for direction in ["inbound", "outbound"]:
                    with sess.get(f"{url}/{direction}?date={date:%Y-%m-%d}") as resp:
                        if "service does not have a timetable" in resp.text:
                            continue
                        if "Service not found" in resp.text:
                            continue
                        timetable = parse_timetable(resp.text)
                        run_times = determine_run_times(timetable)
                        output_timetable(DATA, code, direction, date, timetable)


def main():
    create_db()
    load()


if __name__ == "__main__":
    main()
