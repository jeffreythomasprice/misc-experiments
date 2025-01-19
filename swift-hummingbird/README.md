```
docker compose --env-file local.env up -d
```

```
docker compose --env-file local.env down -v
```

```
docker exec -it db psql -P pager=off experiment --username=user
```

```
watchexec -r swift run
```