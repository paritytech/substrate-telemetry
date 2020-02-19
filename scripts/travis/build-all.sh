#!/usr/bin/env bash

wd="$(dirname $0)"

# Make sure the other scripts can use relative paths if they want to
cd "$wd"

bash ./scripts/travis/build-backend.sh
bash ./scripts/travis/build-frontend.sh
