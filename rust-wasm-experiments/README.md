```
cd client
trunk serve
```

```
cd server
cargo watch -x run
```

```
cargo clippy --fix --allow-dirty --allow-staged && cargo fmt
```