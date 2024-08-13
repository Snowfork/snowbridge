FROM node:20 AS base
FROM base AS build
RUN corepack enable
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY . .
RUN pnpm build

FROM build

VOLUME /env
ENV DOTENV_CONFIG_PATH=/env/.env

WORKDIR /app/packages/operations
COPY --from=build /app/packages/operations/node_modules /app/packages/operations/node_modules
COPY --from=build /app/packages/operations/dist /app/packages/operations/dist

CMD ["node", "./dist/src/cron.js"]

