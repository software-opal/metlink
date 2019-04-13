import requests
from cachecontrol import CacheControl
from cachecontrol.adapter import CacheControlAdapter
from cachecontrol.caches import FileCache
from cachecontrol.heuristics import ExpiresAfter

adapter = CacheControlAdapter(heuristic=ExpiresAfter(days=1))


def get_session():
    cache = FileCache(".web_cache")
    session = requests.Session()
    session.headers.update({"x-api-key": "something-something-darkside"})
    session.mount(
        "https://www.metlink.org.nz/",
        CacheControlAdapter(heuristic=ExpiresAfter(days=7), cache=cache),
    )
    session.mount(
        "https://www.metlink.org.nz/api/v1/ServiceLocation/",
        CacheControlAdapter(heuristic=ExpiresAfter(seconds=90), cache=cache),
    )
    return session
