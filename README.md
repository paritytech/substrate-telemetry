# Polkadot Telemetry

### Setup

```
yarn
yarn start:backend
yarn start:frontend
```

### Use docker image

```
docker run --name polkadot-telemetry-backend -p 1024:1024 -p 8080:8080 --rm -d chevdor/polkadot-telemetry:latest yarn start:backend
docker run --name polkadot-telemetry-frontend -p 3000:3000 --rm -d chevdor/polkadot-telemetry:latest yarn start:frontend

```

### Build docker image

```
docker build -t chevdor/polkadot-telemetry:latest .
```