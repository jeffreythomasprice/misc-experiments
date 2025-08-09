TODO make lib build auto copy the result to the right place

```
cd lib
cargo build
cd ..

mkdir -p executable/priv
cp ./lib/target/debug/rust_lib.dll executable/priv

cd executable
gleam run
```