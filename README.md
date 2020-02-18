# Polkadot Telemetry

## Getting Started
To run the backend, you will need `cargo` to build the binary. We recommend using [`rustup`](https://rustup.rs/).

To run the frontend make sure to grab the latest stable version of node and install dependencies before doing anything:

```
nvm install stable
yarn
```

### Terminal 1 - Backend
```
cd backend
cargo build --release
./target/release/telemetry
```
### Terminal 2 - Frontend
```
yarn start:frontend
```

### Terminal 3 - Node
Follow up installation instructions from the [Polkadot repo](https://github.com/paritytech/polkadot)

```
./target/release/polkadot --dev --telemetry-url ws://localhost:8000/submit
```

### Run via Docker
To run via docker make sure that you have Docker Desktop
  - If you dont you can download for you OS here [Docker Desktop](https://www.docker.com/products/docker-desktop)
```
docker-compose up -d
```
 - -d stands for detach, if you would like to see logs i recommend using [Kitmatic](https://kitematic.com/) or dont use the -d
  - If you want to makes UI changes, there is no need to rebuild the image as the files are being copied in via volumes.

Now navigate to localhost:3000 in your browser to view the app.

### Provision VM for internet use

Currently terraform files are only available for DigitalOcean.

Copy the `tfvars` 
```bash 
cd terraform 

cp terraform.tfvars.sample terraform.tfvars
```

then continue

```bash
terraform init
terraform plan -out=plan
terraform apply plan
```

### Output 

Frontend is available on the IP address you see in the output and on port `5000`

Backend is available on the same IP and port `8000`

Make sure that you add `ws://IP:8000/submit` when you start the polkadot