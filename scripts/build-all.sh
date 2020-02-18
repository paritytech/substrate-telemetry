#!/usr/bin/env bash

wd="$(dirname $0)"

# Make sure the other scripts can use relative paths if they want to
cd "$wd"

bash ./scripts/build-backend.sh
bash ./scripts/build-frontend.sh
