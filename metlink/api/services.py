from ..session import get_session
from . import API_V1_BASE
from .db import Service, create_db, db_session


# The caps come from the JSON, don't change them <3
def load_service(
    *, Code, Name, Mode, Link, LastModified, AliasNames, TrimmedCode, **kwargs
):
    if Code != TrimmedCode:
        print(
            f"Warning(Svc {Code}): Code/TrimmedCode({Code}/{TrimmedCode}) are not the same"
        )
    del TrimmedCode
    if kwargs:
        print(f"Warning(Svc {Code}): extra stop arguments recieved: {list(kwargs)}")

    return Service(
        code=Code,
        name=Name,
        mode=Mode,
        link=Link,
        last_modified=LastModified,
        schools_str=AliasNames,
    )


def import_services():
    with get_session().get(f"{API_V1_BASE}/ServiceList/") as resp:
        resp.raise_for_status()
        data = resp.json()
    with db_session() as session:
        session.add_all(list(filter(None, (load_service(**stop) for stop in data))))


def main():
    create_db()


if __name__ == "__main__":
    main()
