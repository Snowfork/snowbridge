FROM node:20 AS base
FROM base AS deps
RUN corepack enable
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY . .
RUN pnpm build


FROM base
WORKDIR /app/packages/operations
COPY --from=deps /app/packages/operations/node_modules /app/packages/operations/node_modules
COPY --from=build /app/packages/operations/dist /app/packages/operations/dist
ENV NODE_ENV production
CMD ["node", "./dist/src/cron.js"]

