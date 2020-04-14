# Polkadot Telemetry

## Overview

This repository contains both the backend ingestion server for Substrate Telemetry as well as the Frontend you typically see running at [telemetry.polkadot.io](https://telemetry.polkadot.io/).

The backend is a Rust project and the frontend is React/Typescript project.

## Getting Started

To run the backend, you will need `cargo` to build the binary. We recommend using [`rustup`](https://rustup.rs/).

To run the frontend make sure to grab the latest stable version of node and install dependencies before doing anything:

```sh
nvm install stable
yarn
```

### Terminal 1 - Backend

```
cd backend
cargo build --release
./target/release/telemetry --help
```

By default, telemetry will listen on the local interface only (127.0.0.1) on port 8000. You may change both those values with the `--listen` flag as shown below:

```
telemetry --listen 0.0.0.0:8888
```

This example listen on all interfaces and on port :8888

### Terminal 2 - Frontend

```sh
cd frontend
yarn
yarn start
```

### Terminal 3 - Node

Follow up installation instructions from the [Polkadot repo](https://github.com/paritytech/polkadot)

```sh
polkadot --dev --telemetry-url ws://localhost:8000/submit
```

### Run via Docker

To run via docker make sure that you have Docker Desktop

- If you don't you can download for you OS here [Docker Desktop](https://www.docker.com/products/docker-desktop)

```sh
docker-compose up --build -d
```

- -d stands for detach, if you would like to see logs i recommend using [Kitmatic](https://kitematic.com/) or don't use the -d
- --build will build the images and rebuild, but this is not required every time
- If you want to makes UI changes, there is no need to rebuild the image as the files are being copied in via volumes.

Now navigate to localhost:3000 in your browser to view the app.
