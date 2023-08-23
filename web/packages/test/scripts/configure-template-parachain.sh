#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

create_agent() {
    local create_agent_call="0x3301"
    send_create_call $BRIDGE_HUB_PARAID $TEMPLATE_PARA_ID "$create_agent_call"
}

create_channel() {
    local create_channel_call="0x3302"
    send_create_call $BRIDGE_HUB_PARAID $TEMPLATE_PARA_ID "$create_channel_call"
}

configure_template_parachain() {
  create_agent
  create_channel
}
