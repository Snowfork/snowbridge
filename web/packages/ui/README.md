# Snowbridge UI

A demo UI for Snowbridge.

### Running with local E2E stack

Run the app in the development mode.

```console
npm start
```

Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

### Publishing to Rococo-Sepolia Bridge

Copy the env-template.
```console
cp .env-template .env.production
```

Edit `.env.production` and set `REACT_APP_INFURA_KEY`.

Run the publish build command.

```console
npm run build
```

Builds the app for production to the `build` folder.
