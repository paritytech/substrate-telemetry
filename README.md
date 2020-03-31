# Polkadot Telemetry

## Getting Started

To run the backend, you will need `cargo` to build the binary. We recommend using [`rustup`](https://rustup.rs/).

To run the frontend make sure to grab the latest stable version of node and install dependencies before doing anything:

```sh
nvm install stable
yarn
```

### Terminal 1 - Backend

```sh
cd backend
cargo build --release
./target/release/telemetry
```

### Terminal 2 - Frontend

```sh
cd frontend
yarn
yarn start
```

### Terminal 3 - Node

Follow up installation instructions from the [Polkadot repo](https://github.com/paritytech/polkadot)

```sh
./target/release/polkadot --dev --telemetry-url ws://localhost:8000/submit
```

### Run via Docker

To run via docker make sure that you have Docker Desktop

- If you dont you can download for you OS here [Docker Desktop](https://www.docker.com/products/docker-desktop)

```sh
docker-compose up --build -d
```

- -d stands for detach, if you would like to see logs i recommend using [Kitmatic](https://kitematic.com/) or don't use the -d
- --build will build the images and rebuild, but this is not required every time
- If you want to makes UI changes, there is no need to rebuild the image as the files are being copied in via volumes.

Now navigate to localhost:3000 in your browser to view the app.
