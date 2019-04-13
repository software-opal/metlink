import decimal
import functools

import simplejson as json
from bs4 import BeautifulSoup

LAT_LON_PRECISION = 5  # About 1.1m at equator
LAT_LON_EXPONENT = decimal.Decimal(10) ** -LAT_LON_PRECISION

_json_args = dict(use_decimal=True)
_json_dump_args = dict(sort_keys=True, **_json_args)
_json_load_args = dict(**_json_args)


json_dumps = functools.partial(json.dumps, separators=(",", ":"), **_json_dump_args)
pretty_json_dumps = functools.partial(json.dumps, indent="  ", **_json_dump_args)
json_loads = functools.partial(json.loads, **_json_load_args)


@functools.lru_cache(2 ** 10)
def decimal_parse(str):
    with decimal.localcontext() as ctx:
        return decimal.Decimal(str).quantize(
            LAT_LON_EXPONENT, rounding=decimal.ROUND_HALF_EVEN
        )


def parse_html(content):
    return BeautifulSoup(content, "html.parser")
