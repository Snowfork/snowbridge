#!/bin/env python3

from pprint import pprint
from math import (ceil, log2)

# Implements the signature sampling described in https://hackmd.io/9OedC7icR5m-in_moUZ_WQ

# The complete samples method.
def samples(ratio_per_validator, validators_length, slash_rate, randao_commit_expiry, signature_use_count):
    randao_biasability = 172.8 * (74+1+randao_commit_expiry) # Based on markov chain analysis

    result = ceil(log2(
        ratio_per_validator * validators_length * (1/slash_rate) * randao_biasability
    ))

    if(signature_use_count > 0):
        result += 1 + 2 * ceil(log2(signature_use_count))

    return result

# The samples method that we use to get the minimum signatures. Run off-chain and set at beefy client initialization.
def samples_static(ratio_per_validator, slash_rate, randao_commit_expiry):
    randao_biasability = 172.8 * (74+1+randao_commit_expiry) # Based on markov chain analysis

    result = ceil(log2(
        ratio_per_validator * (1/slash_rate) * randao_biasability
    ))

    return result

# The samples method that we used to get the dynamic signatures. Run on-chain.
def samples_dynamic(validators_length, signature_use_count):
    result = ceil(log2(validators_length))

    if(signature_use_count > 0):
      result += 1 + 2 * ceil(log2(signature_use_count))

    return result

pprint([(i, samples(2.5, 1000, 0.25, 3, i), samples_static(2.5, 0.25, 3), samples_dynamic(1000, i)) for i in range(0,2**5)])

