#!/usr/bin/env bash

cd `git rev-parse --show-toplevel`

IMAGE=telemetry-frontend
DOCKER_USER=${DOCKER_USER:-paritytech} 
echo "Publishing $IMAGE as $DOCKER_USER"

docker build -t $IMAGE -f packages/frontend/Dockerfile .
docker tag $IMAGE $DOCKER_USER/$IMAGE
docker push $DOCKER_USER/$IMAGE
