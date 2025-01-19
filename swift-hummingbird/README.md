```
rm -f keys/*{.key,.pub}

# TODO no rsa?
openssl genrsa -out keys/jwt.key 2048
openssl rsa -pubout -in keys/jwt.key -out keys/jwt.key.pub
openssl rsa -text -in keys/jwt.key
openssl rsa -pubin -text -in keys/jwt.key.pub

openssl ecparam -out keys/jwt.key -outform PEM -name prime256v1 -genkey
openssl ecparam -text -in keys/jwt.key
openssl ec -in keys/jwt.key -pubout -out keys/jwt.key.pub
openssl ec -pubin -text -in keys/jwt.key.pub
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