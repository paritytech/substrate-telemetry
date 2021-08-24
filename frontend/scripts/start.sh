#!/usr/bin/env bash

echo Generate env script...
/usr/local/bin/env.sh
echo done

echo Starting nginx...
nginx -g "daemon off;"
