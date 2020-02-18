#!/usr/bin/env bash

cd ./backend

docker build -t woss/polkadot-telemetry-backend:0.1.0 .

docker tag woss/polkadot-telemetry-backend:0.1.0 woss/polkadot-telemetry-backend:latest

docker push woss/polkadot-telemetry-backend:0.1.0

docker push woss/polkadot-telemetry-backend:latest
