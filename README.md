# Sample web application using Axum and Scylla

### Start Scylla DB using Docker

```
$ docker run --name hello-scylla -d scylladb/scylla
```

Get IP address:
```
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' hello-scylla 
```

### Build and run demo app

Option 1: Via docker
```
$ docker build -t rust-axum-scylla .
$ docker run --rm -it rust-axum-scylla --addr <scylla_ip_addr>
```

Option 2: Directly via Cargo
```
$ RUST_LOG=hello=debug,tower_http::trace=debug cargo run -- --addr <scylla_ip_addr>
```

### Test Rest API

Create vehicles:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin1","engine":"Combustion"}'
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin2","engine":"Ev", ev_data: {"battery_capacity_in_kwh": 62, "soc_in_percent": 74}}
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin3","engine":"Phev"}'
```

Find vehicles by vin:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -G --data-urlencode 'vin=vin2'
```

### Check database

```
$ docker exec -it hello-scylla nodetool status
$ docker exec -it hello-scylla cqlsh
cqlsh> USE hello;
cqlsh:hello> SELECT * from vehicles;
```
