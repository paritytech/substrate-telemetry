#Polkadot Telemetry

## Getting Started
After cloning the repo, make sure to grab the latest stable version of node and install dependencies before doing anything.

```
nvm install stable
yarn
```

### Terminal 1 - Backend
```
yarn start:backend
```
### Terminal 2 - Frontend
```
yarn start:frontend
```

### Terminal 3 - Node
```
./target/debug/polkadot --dev --telemetry-url ws://localhost:1024
```

Now navigate to localhost:3000 in your browser to view the app.

### Use docker image

```
docker run --name polkadot-telemetry-backend -p 1024:1024 -p 8080:8080 --rm -d chevdor/polkadot-telemetry:latest yarn start:backend
docker run --name polkadot-telemetry-frontend -p 3000:3000 --rm -d chevdor/polkadot-telemetry:latest yarn start:frontend

```

### Build docker image

```
docker build -t chevdor/polkadot-telemetry:latest .
```
