#!make

.PHONY: init
init:
	command -v forge > /dev/null 2>&1 || bash -c "curl -L https://foundry.paradigm.xyz | bash"
	forge install --no-commit --no-git https://github.com/dapphub/ds-test
	forge install --no-commit --no-git https://github.com/foundry-rs/forge-std
	npm install -g ts-node
	
.PHONY: test
test: 
	forge test -vvvv
