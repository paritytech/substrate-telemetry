#### BUILDER IMAGE  ####

FROM node:12 as builder
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry frontend builder image"

WORKDIR /opt/build

COPY . .
RUN yarn install && \
    yarn build && \
    yarn cache clean

#### OUTPUT IMAGE ####

FROM nginx:stable-alpine
LABEL maintainer="Daniel Maricic daniel@woss.io"
LABEL version="0.1.0"
LABEL description="Polkadot Telemetry frontend"

ENV SUBSTRATE_TELEMETRY_URL=

WORKDIR /usr/share/nginx/html

COPY packages/frontend/env.sh .
RUN apk add --no-cache bash; chmod +x env.sh

COPY nginx/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /opt/build/packages/frontend/build /usr/share/nginx/html

EXPOSE 80

CMD ["/bin/bash", "-c", "/usr/share/nginx/html/env.sh && nginx -g \"daemon off;\""]
