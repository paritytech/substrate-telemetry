#!/usr/bin/env bash

cd ./frontend

docker build -t woss/polkadot-telemetry-frontend:0.1.0 .

docker tag woss/polkadot-telemetry-frontend:0.1.0 woss/polkadot-telemetry-frontend:latest

docker push woss/polkadot-telemetry-frontend:0.1.0

docker push woss/polkadot-telemetry-frontend:latest
