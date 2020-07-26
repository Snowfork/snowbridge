#!/bin/bash

set -euxo pipefail

webapp_dir=${WEBAPP_DIR}

# build webapp
pushd $webapp_dir
yarn
yarn clean
NODE_ENV=production yarn build:www
tar -C packages/apps/build -czf /tmp/webapp-bundle.tar.gz
popd

# deploy webapp
pushd ansible
ansible-playbook webapp.yml --extra-vars webapp_bundle=/tmp/webapp-bundle.tar.gz
popd
