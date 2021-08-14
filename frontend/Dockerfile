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

# When we run the built docker image, the default command uses this
# env var to tell the UI where to look to get feed info.
ENV SUBSTRATE_TELEMETRY_URL=

WORKDIR /usr/share/nginx/html

COPY --from=builder /opt/builder/env.sh /usr/bin/
RUN apk add --no-cache bash; chmod +x /usr/bin/env.sh

COPY --from=builder /opt/builder/nginx/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /opt/builder/nginx/default /etc/nginx/sites-available/default
COPY --from=builder /opt/builder/build /usr/share/nginx/html

EXPOSE 80

CMD ["/bin/bash", "-c", "/usr/bin/env.sh && nginx -g \"daemon off;\""]
