# Must be ran from ../

#############
## Builder ##
#############
FROM node:latest AS build

WORKDIR /bulbasaur
COPY ./bulbasaur/src ./src
COPY ./bulbasaur/static ./static
COPY ./bulbasaur/package.json .
COPY ./bulbasaur/yarn.lock .
COPY ./bulbasaur/svelte.config.js .
COPY ./bulbasaur/tsconfig.json .
COPY ./bulbasaur/vite.config.ts .

RUN yarn install --frozen-lockfile
RUN yarn build

#################
## Final image ##
#################
FROM gcr.io/distroless/nodejs22:latest

WORKDIR /bulbasaur
COPY --from=build /bulbasaur/build ./build
COPY --from=build /bulbasaur/node_modules ./node_modules
COPY --from=build /bulbasaur/package.json .

CMD ["--env-file=../.env", "build"]
