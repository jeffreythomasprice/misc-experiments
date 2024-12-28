```
docker compose --env-file local.env up -d
```

```
cargo watch -x run
```

```
docker compose down -v
```

```
docker exec -it rust-sqlx-postgres-db-1 psql -P pager=off experiment --username=user
```