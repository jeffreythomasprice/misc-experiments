```
docker compose up -d
```

```
docker compose down -v
```

```
docker exec -it broker /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
```

```
cargo run -- --bootstrap-servers localhost:9092  consumer -t foo -t bar
```

```
cargo run -- --bootstrap-servers localhost:9092 producer -t foo -m "Hello, World!"
```