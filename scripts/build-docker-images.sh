#!/usr/bin/env bash

wd="$(dirname $0)"

# Make sure the other scripts can use relative paths if they want to
cd "$wd"


echo "Building docker images for the backend"

cd ./backend

docker build -t woss/polkadot-telemetry-backend:0.1.0 .

docker tag woss/polkadot-telemetry-backend:0.1.0 woss/polkadot-telemetry-backend:latest

docker push woss/polkadot-telemetry-backend:0.1.0

docker push woss/polkadot-telemetry-backend:latest

# get back to the root
cd "$wd"


echo "Building docker images for the frontend"
cd ./frontend

docker build -t woss/polkadot-telemetry-frontend:0.1.0 .

docker tag woss/polkadot-telemetry-frontend:0.1.0 woss/polkadot-telemetry-frontend:latest

docker push woss/polkadot-telemetry-frontend:0.1.0

docker push woss/polkadot-telemetry-frontend:latest
