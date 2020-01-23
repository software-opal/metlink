import decimal
import functools
import pathlib
import re

import simplejson as json
from bs4 import BeautifulSoup

BASE = pathlib.Path(__file__).parent / "../"

LAT_LON_PRECISION = 10
LAT_LON_EXPONENT = decimal.Decimal(10) ** -LAT_LON_PRECISION

_json_args = dict(use_decimal=True)
_json_dump_args = dict(sort_keys=True, **_json_args)
_json_load_args = dict(**_json_args)


json_dumps = functools.partial(json.dumps, separators=(",", ":"), **_json_dump_args)
json_dump = functools.partial(json.dump, separators=(",", ":"), **_json_dump_args)
pretty_json_dumps = functools.partial(json.dumps, indent="  ", **_json_dump_args)
pretty_json_dump = functools.partial(json.dump, indent="  ", **_json_dump_args)
json_loads = functools.partial(json.loads, **_json_load_args)
json_load = functools.partial(json.load, **_json_load_args)


@functools.lru_cache(2 ** 10)
def decimal_parse(str):
    with decimal.localcontext():
        return decimal.Decimal(str).quantize(
            LAT_LON_EXPONENT, rounding=decimal.ROUND_HALF_EVEN
        )


def save_response(resp):
    file = re.sub(r"[^\w\-_\. ]", "_", resp.url)
    if "json" in resp.headers["content-type"]:
        file += ".json"
    elif "html" in resp.headers["content-type"]:
        file += ".html"
    else:
        file += ".txt"
    (BASE / "responses").mkdir(parents=True, exist_ok=True)
    with (BASE / "responses" / file).open("wb") as f:
        f.write(resp.content)


def parse_html(content):
    return BeautifulSoup(content, "html.parser")
