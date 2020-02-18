cd ./frontend

docker build -t woss/polkadot-telemetry-frontend:0.1.0 .

docker tag docker build -t woss/polkadot-telemetry-frontend:0.1.0 docker build -t woss/polkadot-telemetry-frontend:latest

docker push woss/polkadot-telemetry-frontend:0.1.0

docker push woss/polkadot-telemetry-frontend:latest
