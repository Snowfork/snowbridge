#!/usr/bin/env ts-node

import { ValidatorSet, createRandomSubset } from "../helpers"
import { ethers } from "ethers"

const setId = parseInt(process.argv[2])
const setSize = parseInt(process.argv[3])
const setIndex = 0
const commitHash = process.argv[4]
const validatorSet = new ValidatorSet(setId, setSize)
const subsetSize = setSize - Math.floor((setSize - 1) / 3)
const subset = createRandomSubset(setSize, subsetSize)
const validator = validatorSet.createSignatureProof(subset[setIndex], commitHash)

const encoder = new ethers.utils.AbiCoder()

process.stdout.write(
    `${encoder.encode(
        ["bytes32", "bytes32[]", "uint256[]", "address", "uint8", "bytes32", "bytes32"],
        [
            validatorSet.root,
            validatorSet.proofs[setIndex],
            subset,
            validator.account,
            validator.v,
            validator.r,
            validator.s,
        ]
    )}`
)
