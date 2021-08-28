# Sample web application using Axum and Scylla

### Start Scylla DB using Docker

```
$ docker run --name hello-scylla -d scylladb/scylla
```

Get IP address:
```
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' hello-scylla 
```

### Run demo app

```
$ RUST_LOG=hello=debug,tower_http::trace=debug cargo run -- --addr <ip_addr>
```

### Test Rest API

Create vehicles:
```
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin1","engine":{"type": "Combustion"}}'
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin2","engine":{"type": "Ev", "battery_capacity_in_kwh": 62, "soc_in_percent": 74}}
$ curl -v -H "Accept: application/json" -H "Content-type: application/json" localhost:3000/vehicles -d '{"vin":"vin3","engine":{"type": "Phev"}}'
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
