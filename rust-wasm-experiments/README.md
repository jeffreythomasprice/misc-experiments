```
pushd client && trunk serve; popd
```

```
cargo watch -x "run --bin server"
```

```
cargo clippy --fix --allow-dirty --allow-staged && cargo fmt
```