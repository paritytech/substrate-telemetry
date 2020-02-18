cd ./backend

docker build -t woss/polkadot-telemetry-backend:0.1.0 .

docker tag docker build -t woss/polkadot-telemetry-backend:0.1.0 docker build -t woss/polkadot-telemetry-backend:latest

docker push woss/polkadot-telemetry-backend:0.1.0

docker push woss/polkadot-telemetry-backend:latest
