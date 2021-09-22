# rust-axum-scylla

Sample web application using [Axum](https://github.com/tokio-rs/axum) and [Scylla](https://www.scylladb.com)

### Start Scylla DB using Docker

```
$ docker run --name hello-scylla -d scylladb/scylla
```

Get IP address:
```
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' hello-scylla 
```

### Build and run demo app

```
$ RUST_LOG=hello=debug,tower_http::trace=debug cargo run -- --addr <scylla_ip_addr>
```

### Test (cargo)

```
$ cargo test
```

### Test (curl)

Create vehicles:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin1","engine_type":"Combustion"}'
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin2","engine_type":"Ev", ev_data: {"battery_capacity_in_kwh": 62, "soc_in_percent": 74}}
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin3","engine_type":"Phev"}'
```

Find vehicles by vin:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -G --data-urlencode 'vin=vin2'
```

### Check database

```
$ docker exec -it hello-scylla nodetool status
$ docker exec -it hello-scylla cqlsh
cqlsh> SELECT * from hello.vehicles;
```

