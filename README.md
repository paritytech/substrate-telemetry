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
