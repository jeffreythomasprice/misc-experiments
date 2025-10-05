```
dune exec -w ./bin/main.exe
```

```
dune exec -w ./test/test.exe
```

List tests:
```
dune exec ./test/test.exe -- -list-test
```

Run specific test:
```
dune exec ./test/test.exe -- -only-test "test suite for whole program:1:test suite for triple:0:test 4"
```