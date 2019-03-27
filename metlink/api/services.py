from .session import get_session
from . import API_V1_BASE
from .db import Session, create_db, Services
import datetime as dt


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

    return Services(
        name=Code,
        code=Name,
        mode=Mode,
        link=Link,
        last_modified=LastModified,
        schools_str=AliasNames,
    )


def main():
    create_db()
    with get_session().get(f"{API_V1_BASE}/ServiceList/") as resp:
        resp.raise_for_status()
        data = resp.json()
    session = Session()
    session.add_all(list(filter(None, (load_service(**stop) for stop in data))))
    session.commit()


if __name__ == "__main__":
    main()
