# rust-axum-scylla

Sample web application using [Axum](https://github.com/tokio-rs/axum) and [Scylla](https://www.scylladb.com)

### Start Scylla DB using Docker

```
$ docker run --name hello-scylla -d -p 9042:9042 scylladb/scylla
```

### Build and run demo app

```
$ RUST_LOG=hello=debug,tower_http::trace=debug cargo run
```

### Test (cargo)

All tests:
```
$ cargo test
```

Unit tests only:
```
$ cargo test --lib
```

Integration tests only:
```
$ cargo test --test '*' -- --test-threads=1
```

Note: we need to ensure that the tests are not concurrently executed because it would mess up the checks.

### Test (curl)

Create vehicles:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -d '{"vin":"vin1","engine_type":"Combustion"}'
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -d '{"vin":"vin2","engine_type":"Ev", ev_data: {"battery_capacity_in_kwh": 62, "soc_in_percent": 74}}
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -d '{"vin":"vin3","engine_type":"Phev"}'
```

Find vehicles by vin:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -G --data-urlencode 'vin=vin2'
```

### Check database

```
$ docker exec -it hello-scylla nodetool status
$ docker exec -it hello-scylla cqlsh
cqlsh> SELECT * from hello.vehicles;
```

