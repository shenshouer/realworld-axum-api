# tools

## db
```
docker run --name realworld-db \
  -e POSTGRES_PASSWORD=realworld123 \
  -e POSTGRES_USER=realworld \
  -e POSTGRES_DB=realworld_dev \
  -p 5432:5432 \
  -d postgres:15
```

```
docker exec -it realworld-db psql -U realworld -d realworld_dev
```

```
sqlx migrate add create_users_table
sqlx migrate run
```

## openobserve

```
docker rm openobserve
docker run --name openobserve \
  -v $PWD/data:/data \
  -e ZO_DATA_DIR="/data" \
  -p 5080:5080 \
  -p 5081:5081 \
  -e ZO_ROOT_USER_EMAIL="root@example.com" \
  -e ZO_ROOT_USER_PASSWORD="Complexpass#123" \
  o2cr.ai/openobserve/openobserve-enterprise:latest

```
