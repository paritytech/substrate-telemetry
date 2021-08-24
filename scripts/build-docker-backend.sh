#!/usr/bin/env bash

pushd "$(git rev-parse --show-toplevel)/backend" > /dev/null

while getopts ":Nsgapv:" arg; do
  case "${arg}" in
    p)
      PUBLISH="true"
      ;;
  esac
done

IMAGE=telemetry-backend
DOCKER_USER=${DOCKER_USER:-paritytech}
echo "Publishing $IMAGE as $DOCKER_USER"

docker build -t $DOCKER_USER/$IMAGE -f ./Dockerfile .

if [[ "$PUBLISH" = 'true' ]]; then
    docker push $DOCKER_USER/$IMAGE
else
    echo 'No -p passed, skipping publishing to docker hub'
fi

popd > /dev/null

docker images | grep $IMAGE
