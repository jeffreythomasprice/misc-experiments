```
rm -f keys/*{.key,.pub}
openssl genrsa -out keys/jwt.key 2048
openssl rsa -pubout -in keys/jwt.key -out keys/jwt.key.pub
openssl rsa -text -in keys/jwt.key
openssl rsa -pubin -text -in keys/jwt.key.pub
```

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