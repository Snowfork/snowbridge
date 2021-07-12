[ethereum]
endpoint = "{{ key "snowbridge/ethereum-provider/endpoint" }}"
startblock = 1
descendants-until-final = 3
beefylightclient = "{{ key "snowbridge/contract/beefy-light-client/address" }}"

[ethereum.channels.basic]
inbound = "{{ key "snowbridge/contract/basic-inbound-channel/address" }}"
outbound = "{{ key "snowbridge/contract/basic-outbound-channel/address" }}"

[ethereum.channels.incentivized]
inbound = "{{ key "snowbridge/contract/incentivized-inbound-channel/address" }}"
outbound = "{{ key "snowbridge/contract/incentivized-outbound-channel/address" }}"

[parachain]
{{ with service "snowbridge-rpc" }}{{ with index . 0 -}}
endpoint = "ws://{{ .Address }}:{{ .Port }}/"
{{- end }}{{ end }}

[relaychain]
{{ with service "polkadot-rpc" }}{{ with index . 0 -}}
endpoint = "ws://{{ .Address }}:{{ .Port }}/"
{{- end }}{{- end }}

[workers.parachaincommitmentrelayer]
enabled = true
restart-delay = 30

[workers.beefyrelayer]
enabled = true
restart-delay = 30

[workers.ethrelayer]
enabled = true
restart-delay = 30
