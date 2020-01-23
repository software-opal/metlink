import time
from .utils import BASE
import requests
from cachecontrol.adapter import CacheControlAdapter
from cachecontrol.caches import FileCache
from cachecontrol.heuristics import ExpiresAfter, LastModified

adapter = CacheControlAdapter(heuristic=ExpiresAfter(days=1))

current_time = time.time

CACHE_FOLDER = BASE / ".web_cache"
RATE_LIMIT_SECONDS = 30
RATE_LIMIT_REQUESTS = 12


class BetterExpiresAfter(ExpiresAfter):
    def update_headers(self, response):
        if response.status not in LastModified.cacheable_by_default_statuses:
            return {}
        return super().update_headers(response)


class DebugFileCache(FileCache):

    def get(self, key):
        print(f"get({key!r})")
        return super().get(key)

    def set(self, key, value):
        print(f"set({key!r}, {value!r})")
        return super().set(key, value)

    def delete(self, key):
        print(f"delete({key!r})")
        return super().delete(key)



class RateLimitingSession(requests.Session):

    def __init__(self, *a, **k):
        super().__init__(*a, **k)
        self.request_times = []

    def send(self, request, *args, retries=0, **kwargs):
        self._wait_request_times()
        self.request_times.append(current_time())
        response = super().send(request, *args, **kwargs)
        if response.status_code == 429:
            from pprint import pprint; pprint(response.url)
            from pprint import pprint; pprint(response.headers)
            if retries < 2:
                if response.headers.get("Retry-After", None):
                    print(f"Retrying request(retry {retries+1}/3): {request.url}")
                    print(f"Request times: {sorted(self.request_times)}")
                    print(f"Current time: {current_time()}")
                    wait_seconds = int(response.headers["Retry-After"])
                    time.sleep(wait_seconds)
                    print(f"Current time: {current_time()}")
                    return self.send(request, *args, retries=retries + 1, **kwargs)
            else:
                print(f"Request failed after {retries+1} attempts: {request.url}")
        elif getattr(response, 'from_cache', False):
            # This request came from the cache so doesn't count towards our rate limit.
            self.request_times.pop()
        return response

    def _wait_request_times(self):
        self.request_times = sorted(self.request_times)
        while len(self.request_times) > RATE_LIMIT_REQUESTS:
            oldest = self.request_times[0]
            now = current_time()
            print(f"Oldest request sent at {oldest}, Next request can be sent at { oldest + RATE_LIMIT_SECONDS}; it is now {now}")
            while oldest + RATE_LIMIT_SECONDS > now:
                time.sleep(min(5, oldest + RATE_LIMIT_SECONDS - now))
                now = current_time()
                print(f" | it is now {now}")
            self.request_times = self.request_times[1:]

def get_session():
    print("AAAA")
    CACHE_FOLDER.mkdir(exist_ok=True)
    cache = FileCache(str(CACHE_FOLDER), forever=True)
    cache.set("foo", b"bar")
    assert cache.get("foo") == b"bar"
    session = RateLimitingSession()
    # session.headers.update({"x-api-key": "something-something-darkside"})
    session.mount(
        "https://www.metlink.org.nz/",
        CacheControlAdapter(heuristic=BetterExpiresAfter(days=7), cache=cache),
    )
    session.mount(
        "https://www.metlink.org.nz/api/v1/",
        CacheControlAdapter(heuristic=BetterExpiresAfter(days=1), cache=cache),
    )
    session.mount(
        "https://www.metlink.org.nz/api/v1/ServiceLocation/",
        CacheControlAdapter(heuristic=BetterExpiresAfter(seconds=90), cache=cache),
    )
    return session
