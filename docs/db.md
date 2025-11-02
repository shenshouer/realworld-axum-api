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
