// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

type ParaID is uint256;

using {paraIDeq as ==, paraIDne as !=, paraIDisNone} for ParaID global;

function paraIDeq(ParaID a, ParaID b) pure returns (bool) {
    return ParaID.unwrap(a) == ParaID.unwrap(b);
}

function paraIDne(ParaID a, ParaID b) pure returns (bool) {
    return !paraIDeq(a, b);
}

function paraIDisNone(ParaID a) pure returns (bool) {
    return ParaID.unwrap(a) == 0;
}

type ChannelID is uint256;

using {channelIDeq as ==, channelIDne as !=} for ChannelID global;

function channelIDeq(ChannelID a, ChannelID b) pure returns (bool) {
    return ChannelID.unwrap(a) == ChannelID.unwrap(b);
}

function channelIDne(ChannelID a, ChannelID b) pure returns (bool) {
    return !channelIDeq(a, b);
}
