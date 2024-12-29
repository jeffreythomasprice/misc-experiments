```
docker compose --env-file local.env up -d
```

```
docker compose --env-file local.env down -v
```

```
cargo watch -x run
```

```
docker exec -it db psql -P pager=off experiment --username=user
```
