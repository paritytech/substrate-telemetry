FROM mhart/alpine-node:10
MAINTAINER "chevdor@gmail.com"

WORKDIR /app

COPY . .

RUN yarn

EXPOSE 8080 1024 3000
CMD ["yarn", "start"]