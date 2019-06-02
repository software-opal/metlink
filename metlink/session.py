import requests
from cachecontrol import CacheControl
from cachecontrol.adapter import CacheControlAdapter
from cachecontrol.caches import FileCache
from cachecontrol.heuristics import ExpiresAfter, LastModified

adapter = CacheControlAdapter(heuristic=ExpiresAfter(days=1))


class BetterExpiresAfter(ExpiresAfter):
    def update_headers(self, response):
        if response.status not in LastModified.cacheable_by_default_statuses:
            return {}
        return super().update_headers(response)


def get_session():
    cache = FileCache(".web_cache")
    session = requests.Session()
    # session.headers.update({"x-api-key": "something-something-darkside"})
    session.mount(
        "https://www.metlink.org.nz/",
        CacheControlAdapter(heuristic=BetterExpiresAfter(days=7), cache=cache),
    )
    session.mount(
        "https://www.metlink.org.nz/api/v1/",
        CacheControlAdapter(heuristic=BetterExpiresAfter(days=1), cache=None),
    )
    session.mount(
        "https://www.metlink.org.nz/api/v1/ServiceLocation/",
        CacheControlAdapter(heuristic=BetterExpiresAfter(seconds=90), cache=cache),
    )
    return session
