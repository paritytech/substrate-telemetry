#!/usr/bin/env bash
set -e

pushd "$(git rev-parse --show-toplevel)/frontend" > /dev/null

while getopts ":Nsgapv:" arg; do
  case "${arg}" in
    p)
      PUBLISH="true"
      ;;
  esac
done

IMAGE=substrate-telemetry-frontend
DOCKER_USER=${DOCKER_USER:-paritytech}
echo "Publishing $IMAGE as $DOCKER_USER"

docker build -t $IMAGE -f ./Dockerfile .
docker tag $IMAGE $DOCKER_USER/$IMAGE

if [[ "$PUBLISH" = 'true' ]]; then
    docker push $DOCKER_USER/$IMAGE
else
    echo 'No -p passed, skipping publishing to docker hub'
fi

popd > /dev/null

docker images | grep $IMAGE
