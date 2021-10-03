# rust-axum-scylla

Sample web application using [Axum](https://github.com/tokio-rs/axum) and [Scylla](https://www.scylladb.com)

### Start Scylla DB using Docker

```
$ docker run --name hello-scylla -d -p 9042:9042 scylladb/scylla
```

### Build and run demo app

Directly via cargo:
```
$ RUST_LOG=hello=debug,tower_http::trace=debug cargo run
```

Via docker:
```
$ docker run -t -i -p 3000:3000 --link=hello-scylla:scylla -it hello-app --addr scylla
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

Create vehicle:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -d '{"vin":"vin1","engine_type":"Combustion"}'
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -d '{"vin":"vin2","engine_type":"Ev", ev_data: {"battery_capacity_in_kwh": 62, "soc_in_percent": 74}}
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicle -d '{"vin":"vin3","engine_type":"Phev"}'
```

Find vehicle by vin:
```
$ curl -v -H "Accept: application/json" localhost:3000/vehicle/vin2 -G
```

Delete vehicle by vin:
```
$ curl -v -H "Accept: application/json" -X DELETE localhost:3000/vehicle/vin2 -G
```

### Check database

```
$ docker exec -it hello-scylla nodetool status
$ docker exec -it hello-scylla cqlsh
cqlsh> SELECT * from hello.vehicles;
```

