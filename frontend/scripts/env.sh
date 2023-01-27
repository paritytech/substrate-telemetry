#!/usr/bin/env bash

# This script is used when the docker container starts and does the magic to
# bring the ENV variables to the generated static UI.

ENV_DIR=./tmp
mkdir -p "$ENV_DIR"
TARGET="$ENV_DIR/env-config.js"

# Recreate config file
echo -n > $TARGET

declare -a vars=(
  "SUBSTRATE_TELEMETRY_URL"
)

echo "window.process_env = {" >> $TARGET
for VAR in ${vars[@]}; do
  echo "  $VAR: \"${!VAR}\"," >> $TARGET
done
echo "}" >> $TARGET

chmod 440 $TARGET
