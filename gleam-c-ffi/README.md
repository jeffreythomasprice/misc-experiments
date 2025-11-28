# Manual

A c library...
```
cd c-lib
make
```

...consumed by an erland NIF...
```
cd c-lib-nif-manual
make
```

...consumed by gleam.
```
cd gleam-manual
LD_LIBRARY_PATH=$(pwd)/priv gleam run
```

# Automatic

TODO docs

```
cd gleam-clang-generator
gleam run
```