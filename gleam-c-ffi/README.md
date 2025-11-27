```
clang -Xclang -ast-dump=json c-lib/src/lib.h
```


```
cd c-lib
make
```

```
cd c-lib-nif-manual
make
```

```
cd gleam-manual
LD_LIBRARY_PATH=$(pwd)/priv gleam run
```
