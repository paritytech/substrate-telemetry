# Polkadot Telemetry

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

### Run via Docker
To run via docker make sure that you have Docker Desktop
  - If you dont you can download for you OS here [Docker Desktop](https://www.docker.com/products/docker-desktop)
```
docker-compose up --build -d
```
 - -d stands for detach, if you would like to see logs i recommend using [Kitmatic](https://kitematic.com/) or dont use the -d
 - --build will build the images and rebuild, but this is not required everytime
  - If you want to makes UI changes, there is no need to rebuild the image as the files are being copied in via volumes.
  
Now navigate to localhost:3000 in your browser to view the app.
