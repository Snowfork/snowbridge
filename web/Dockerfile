FROM node:22
RUN corepack enable
COPY . /app
WORKDIR /app

RUN rm -rf /app/packages/contracts && rm -rf /app/packages/test && rm -rf /app/packages/test-helpers

WORKDIR /app/packages/operations
VOLUME /config
ENV DOTENV_CONFIG_PATH=/config/.env
ENTRYPOINT [ "node", "./dist/src/main.js" ]
CMD [ "start" ]

