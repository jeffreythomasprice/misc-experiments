```
docker compose up -d
```

```
docker compose down -v
```

```
cargo run -- --bootstrap-servers localhost:9092  consumer -t foo -t bar
```

```
cargo run -- --bootstrap-servers localhost:9092 producer -t foo -m "Hello, World!"
```