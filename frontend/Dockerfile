#### BUILDER IMAGE  ####
FROM docker.io/node:12 as builder
LABEL maintainer="Chevdor <chevdor@gmail.com>"
LABEL description="Polkadot Telemetry frontend builder image"

WORKDIR /opt/builder

COPY . .

RUN yarn install && \
    yarn build && \
    yarn cache clean

#### OUTPUT IMAGE ####
FROM docker.io/nginx:stable-alpine
LABEL maintainer="Chevdor <chevdor@gmail.com>"
LABEL description="Polkadot Telemetry frontend"

# Each time this container is ran, the value that's provided for this env var
# determines where the frontend will try to request feed information from:
ENV SUBSTRATE_TELEMETRY_URL=

WORKDIR /app

COPY --from=builder /opt/builder/scripts/*.sh /usr/local/bin/
COPY --from=builder /opt/builder/build /app

RUN chown -R nginx:nginx /app && \
    chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chown -R nginx:nginx /etc/nginx/conf.d

RUN touch /var/run/nginx.pid && \
    chown -R nginx:nginx /var/run/nginx.pid

RUN apk add --no-cache bash

COPY --from=builder /opt/builder/nginx/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /opt/builder/build /app

# UID= 101
USER nginx
EXPOSE 8000

CMD ["/usr/local/bin/start.sh"]
