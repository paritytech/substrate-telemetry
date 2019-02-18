FROM node:alpine

WORKDIR /app

RUN apk update && apk add python g++ make openssh git bash pdftk
RUN export PYTHONPATH=${PYTHONPATH}:/usr/lib/python2.7

COPY ./package.json yarn.lock ./

RUN yarn 

COPY . .

# Frontend is exposing 3000
# Backend is exposing 8080
# No need for expose, if using docker-compose & docker run -p 3000:3000
