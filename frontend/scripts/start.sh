#!/usr/bin/env sh

ENV_CONFIG=/app/tmp/env-config.js

if test -f $ENV_CONFIG; then
    echo Config is locked
else
    echo Generate env-config script...
    /usr/local/bin/env.sh
    echo done
    chmod 444 $ENV_CONFIG
fi

echo Starting nginx...
nginx -g "daemon off;"
