import requests
from cachecontrol import CacheControl
from cachecontrol.caches import FileCache


def get_session():
    cache = FileCache(".web_cache")
    return CacheControl(requests.Session(), cache)
