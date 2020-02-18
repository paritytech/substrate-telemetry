#!/usr/bin/env bash

echo "Cloning repository"
git clone https://github.com/woss/substrate-telemetry.git /app

cd /app

echo "Switching to another branch"
git checkout docker-improvements

echo "Executing docker-compose"
docker-compose up -d
