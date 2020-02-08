# Metlink data explorer

## Why?
Because I wanted to get the current locations of each bus, and generate a route map for them for pretty pictures

### Metlink's service maps

Due to the way that Metlink returns the route maps, it does not map nicely to a timetabled route:

```
  /---- B ------- C
 /
A -- X -- D -- Y -- E
           \
            \-- Z -- F
```
For a given route that runs from stop `A`, to either `C` and `E` or `F`; there will be four route maps:
- `A` to `C`
- `A` to `D`
- `D` to `E`
- `D` to `F`

We can use the timetables to determine which combinations of route maps create actually travelled routes.

## Setting up

```
poetry install
cargo build --all
```

## Running the progams

### Downloading API content

This will take a long time, it needs to download every route, and all the timetables, whilst working within the API request limiting of 5 requests every 30 seconds(ish)
```
poetry run python -m metlink
```

### Generating route maps


To generate these you need to have the stops, timetables and services downloaded into `data`. Then run:

```
cargo run --package metlink-route-builder
```

This will generate `routes.json` files for each service.
