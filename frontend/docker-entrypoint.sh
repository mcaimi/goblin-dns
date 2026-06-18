#!/bin/sh
set -e

# Set default backend URL if not provided
BACKEND_URL=${BACKEND_URL:-"http://localhost:8080"}

# Substitute backend URL in nginx config
envsubst '${BACKEND_URL}' < /etc/nginx/conf.d/default.conf > /etc/nginx/conf.d/default.conf.tmp
mv /etc/nginx/conf.d/default.conf.tmp /etc/nginx/conf.d/default.conf

# Start nginx
exec nginx -g 'daemon off;'
