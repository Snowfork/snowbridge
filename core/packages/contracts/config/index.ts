interface EnvironmentConfig {
    parachainID: number,
    randaoCommitDelay: number,
    randaoCommitExpiration: number,
}

interface Config {
    [network: string]: EnvironmentConfig;
}

const config: Config = {
    default: {
        parachainID: 1000,
        randaoCommitDelay: 3,
        randaoCommitExpiration: 8
    }
}

const getConfigForNetwork = (network: string): EnvironmentConfig => {
    if (network in config) {
        return config[network]
    }
    return config.default
}

export { getConfigForNetwork }
