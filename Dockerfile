FROM node:16-alpine as build

WORKDIR /builder

COPY package.json yarn.lock ./

RUN yarn install --pure-lockfile

COPY . .

RUN yarn build

FROM node:16-alpine

ENV NODE_ENV=production
ENV PORT=3000
ENV CACHE_DIR=/cache

RUN mkdir -p $CACHE_DIR

WORKDIR /app

COPY package.json yarn.lock ./

RUN yarn install --pure-lockfile

COPY --from=build /builder/dist ./dist

EXPOSE ${PORT}

CMD [ "node", "dist/index.js" ]