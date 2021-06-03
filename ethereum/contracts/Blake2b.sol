// SPDX-License-Identifier: Apache-2.0
// from https://github.com/ConsenSys/Project-Alchemy/blob/master/contracts/BLAKE2b/BLAKE2b.sol
pragma solidity ^0.7.0;

contract Blake2b {
    uint64[8] public IV = [
        0x6a09e667f3bcc908,
        0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b,
        0xa54ff53a5f1d36f1,
        0x510e527fade682d1,
        0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b,
        0x5be0cd19137e2179
    ];

    uint64 constant MASK_0 = 0xFF00000000000000;
    uint64 constant MASK_1 = 0x00FF000000000000;
    uint64 constant MASK_2 = 0x0000FF0000000000;
    uint64 constant MASK_3 = 0x000000FF00000000;
    uint64 constant MASK_4 = 0x00000000FF000000;
    uint64 constant MASK_5 = 0x0000000000FF0000;
    uint64 constant MASK_6 = 0x000000000000FF00;
    uint64 constant MASK_7 = 0x00000000000000FF;

    uint64 constant SHIFT_0 = 0x0100000000000000;
    uint64 constant SHIFT_1 = 0x0000010000000000;
    uint64 constant SHIFT_2 = 0x0000000001000000;
    uint64 constant SHIFT_3 = 0x0000000000000100;

    struct Blake2b_ctx {
        uint256[4] b; //input buffer
        uint64[8] h; //chained state
        uint128 t; //total bytes
        uint64 c; //Size of b
        uint256 outlen; //diigest output size
    }

    // Mixing Function
    function G(
        uint64[16] memory v,
        uint256 a,
        uint256 b,
        uint256 c,
        uint256 d,
        uint64 x,
        uint64 y
    ) public pure {
        // Dereference to decrease memory reads
        uint64 va = v[a];
        uint64 vb = v[b];
        uint64 vc = v[c];
        uint64 vd = v[d];

        //Optimised mixing function
        assembly {
            // v[a] := (v[a] + v[b] + x) mod 2**64
            va := addmod(add(va, vb), x, 0x10000000000000000)
            //v[d] := (v[d] ^ v[a]) >>> 32
            vd := xor(div(xor(vd, va), 0x100000000), mulmod(xor(vd, va), 0x100000000, 0x10000000000000000))
            //v[c] := (v[c] + v[d])     mod 2**64
            vc := addmod(vc, vd, 0x10000000000000000)
            //v[b] := (v[b] ^ v[c]) >>> 24
            vb := xor(div(xor(vb, vc), 0x1000000), mulmod(xor(vb, vc), 0x10000000000, 0x10000000000000000))
            // v[a] := (v[a] + v[b] + y) mod 2**64
            va := addmod(add(va, vb), y, 0x10000000000000000)
            //v[d] := (v[d] ^ v[a]) >>> 16
            vd := xor(div(xor(vd, va), 0x10000), mulmod(xor(vd, va), 0x1000000000000, 0x10000000000000000))
            //v[c] := (v[c] + v[d])     mod 2**64
            vc := addmod(vc, vd, 0x10000000000000000)
            // v[b] := (v[b] ^ v[c]) >>> 63
            vb := xor(div(xor(vb, vc), 0x8000000000000000), mulmod(xor(vb, vc), 0x2, 0x10000000000000000))
        }

        v[a] = va;
        v[b] = vb;
        v[c] = vc;
        v[d] = vd;
    }

    function compress(Blake2b_ctx memory ctx, bool last) internal view {
        //TODO: Look into storing these as uint256[4]
        uint64[16] memory v;
        uint64[16] memory m;

        for (uint256 i = 0; i < 8; i++) {
            v[i] = ctx.h[i]; // v[:8] = h[:8]
            v[i + 8] = IV[i]; // v[8:] = IV
        }

        //
        v[12] = v[12] ^ uint64(ctx.t % 2**64); //Lower word of t
        v[13] = v[13] ^ uint64(ctx.t / 2**64);

        if (last) v[14] = ~v[14]; //Finalization flag

        uint64 mi; //Temporary stack variable to decrease memory ops
        uint256 b; // Input buffer

        for (uint256 i = 0; i < 16; i++) {
            //Operate 16 words at a time
            uint256 k = i % 4; //Current buffer word
            mi = 0;
            if (k == 0) {
                b = ctx.b[i / 4]; //Load relevant input into buffer
            }

            //Extract relevent input from buffer
            assembly {
                mi := and(div(b, exp(2, mul(64, sub(3, k)))), 0xFFFFFFFFFFFFFFFF)
            }

            //Flip endianness
            m[i] = getWords(mi);
        }

        //Mix m

        G(v, 0, 4, 8, 12, m[0], m[1]);
        G(v, 1, 5, 9, 13, m[2], m[3]);
        G(v, 2, 6, 10, 14, m[4], m[5]);
        G(v, 3, 7, 11, 15, m[6], m[7]);
        G(v, 0, 5, 10, 15, m[8], m[9]);
        G(v, 1, 6, 11, 12, m[10], m[11]);
        G(v, 2, 7, 8, 13, m[12], m[13]);
        G(v, 3, 4, 9, 14, m[14], m[15]);

        G(v, 0, 4, 8, 12, m[14], m[10]);
        G(v, 1, 5, 9, 13, m[4], m[8]);
        G(v, 2, 6, 10, 14, m[9], m[15]);
        G(v, 3, 7, 11, 15, m[13], m[6]);
        G(v, 0, 5, 10, 15, m[1], m[12]);
        G(v, 1, 6, 11, 12, m[0], m[2]);
        G(v, 2, 7, 8, 13, m[11], m[7]);
        G(v, 3, 4, 9, 14, m[5], m[3]);

        G(v, 0, 4, 8, 12, m[11], m[8]);
        G(v, 1, 5, 9, 13, m[12], m[0]);
        G(v, 2, 6, 10, 14, m[5], m[2]);
        G(v, 3, 7, 11, 15, m[15], m[13]);
        G(v, 0, 5, 10, 15, m[10], m[14]);
        G(v, 1, 6, 11, 12, m[3], m[6]);
        G(v, 2, 7, 8, 13, m[7], m[1]);
        G(v, 3, 4, 9, 14, m[9], m[4]);

        G(v, 0, 4, 8, 12, m[7], m[9]);
        G(v, 1, 5, 9, 13, m[3], m[1]);
        G(v, 2, 6, 10, 14, m[13], m[12]);
        G(v, 3, 7, 11, 15, m[11], m[14]);
        G(v, 0, 5, 10, 15, m[2], m[6]);
        G(v, 1, 6, 11, 12, m[5], m[10]);
        G(v, 2, 7, 8, 13, m[4], m[0]);
        G(v, 3, 4, 9, 14, m[15], m[8]);

        G(v, 0, 4, 8, 12, m[9], m[0]);
        G(v, 1, 5, 9, 13, m[5], m[7]);
        G(v, 2, 6, 10, 14, m[2], m[4]);
        G(v, 3, 7, 11, 15, m[10], m[15]);
        G(v, 0, 5, 10, 15, m[14], m[1]);
        G(v, 1, 6, 11, 12, m[11], m[12]);
        G(v, 2, 7, 8, 13, m[6], m[8]);
        G(v, 3, 4, 9, 14, m[3], m[13]);

        G(v, 0, 4, 8, 12, m[2], m[12]);
        G(v, 1, 5, 9, 13, m[6], m[10]);
        G(v, 2, 6, 10, 14, m[0], m[11]);
        G(v, 3, 7, 11, 15, m[8], m[3]);
        G(v, 0, 5, 10, 15, m[4], m[13]);
        G(v, 1, 6, 11, 12, m[7], m[5]);
        G(v, 2, 7, 8, 13, m[15], m[14]);
        G(v, 3, 4, 9, 14, m[1], m[9]);

        G(v, 0, 4, 8, 12, m[12], m[5]);
        G(v, 1, 5, 9, 13, m[1], m[15]);
        G(v, 2, 6, 10, 14, m[14], m[13]);
        G(v, 3, 7, 11, 15, m[4], m[10]);
        G(v, 0, 5, 10, 15, m[0], m[7]);
        G(v, 1, 6, 11, 12, m[6], m[3]);
        G(v, 2, 7, 8, 13, m[9], m[2]);
        G(v, 3, 4, 9, 14, m[8], m[11]);

        G(v, 0, 4, 8, 12, m[13], m[11]);
        G(v, 1, 5, 9, 13, m[7], m[14]);
        G(v, 2, 6, 10, 14, m[12], m[1]);
        G(v, 3, 7, 11, 15, m[3], m[9]);
        G(v, 0, 5, 10, 15, m[5], m[0]);
        G(v, 1, 6, 11, 12, m[15], m[4]);
        G(v, 2, 7, 8, 13, m[8], m[6]);
        G(v, 3, 4, 9, 14, m[2], m[10]);

        G(v, 0, 4, 8, 12, m[6], m[15]);
        G(v, 1, 5, 9, 13, m[14], m[9]);
        G(v, 2, 6, 10, 14, m[11], m[3]);
        G(v, 3, 7, 11, 15, m[0], m[8]);
        G(v, 0, 5, 10, 15, m[12], m[2]);
        G(v, 1, 6, 11, 12, m[13], m[7]);
        G(v, 2, 7, 8, 13, m[1], m[4]);
        G(v, 3, 4, 9, 14, m[10], m[5]);

        G(v, 0, 4, 8, 12, m[10], m[2]);
        G(v, 1, 5, 9, 13, m[8], m[4]);
        G(v, 2, 6, 10, 14, m[7], m[6]);
        G(v, 3, 7, 11, 15, m[1], m[5]);
        G(v, 0, 5, 10, 15, m[15], m[11]);
        G(v, 1, 6, 11, 12, m[9], m[14]);
        G(v, 2, 7, 8, 13, m[3], m[12]);
        G(v, 3, 4, 9, 14, m[13], m[0]);

        G(v, 0, 4, 8, 12, m[0], m[1]);
        G(v, 1, 5, 9, 13, m[2], m[3]);
        G(v, 2, 6, 10, 14, m[4], m[5]);
        G(v, 3, 7, 11, 15, m[6], m[7]);
        G(v, 0, 5, 10, 15, m[8], m[9]);
        G(v, 1, 6, 11, 12, m[10], m[11]);
        G(v, 2, 7, 8, 13, m[12], m[13]);
        G(v, 3, 4, 9, 14, m[14], m[15]);

        G(v, 0, 4, 8, 12, m[14], m[10]);
        G(v, 1, 5, 9, 13, m[4], m[8]);
        G(v, 2, 6, 10, 14, m[9], m[15]);
        G(v, 3, 7, 11, 15, m[13], m[6]);
        G(v, 0, 5, 10, 15, m[1], m[12]);
        G(v, 1, 6, 11, 12, m[0], m[2]);
        G(v, 2, 7, 8, 13, m[11], m[7]);
        G(v, 3, 4, 9, 14, m[5], m[3]);

        //XOR current state with both halves of v
        for (uint256 i = 0; i < 8; ++i) {
            ctx.h[i] = ctx.h[i] ^ v[i] ^ v[i + 8];
        }
    }

    function init(
        Blake2b_ctx memory ctx,
        uint64 outlen,
        bytes memory key,
        uint64[2] memory salt,
        uint64[2] memory person
    ) internal view {
        if (outlen == 0 || outlen > 64 || key.length > 64) revert();

        //Initialize chained-state to IV
        for (uint256 j = 0; j < 8; j++) {
            ctx.h[j] = IV[j];
        }

        // Set up parameter block
        ctx.h[0] = ctx.h[0] ^ 0x01010000 ^ shift_left(uint64(key.length), 8) ^ outlen;
        ctx.h[4] = ctx.h[4] ^ salt[0];
        ctx.h[5] = ctx.h[5] ^ salt[1];
        ctx.h[6] = ctx.h[6] ^ person[0];
        ctx.h[7] = ctx.h[7] ^ person[1];

        ctx.outlen = outlen;

        //Run hash once with key as input
        if (key.length > 0) {
            update(ctx, key);
            ctx.c = 128;
        }
    }

    function update(Blake2b_ctx memory ctx, bytes memory input) internal view {
        for (uint256 i = 0; i < input.length; i++) {
            //If buffer is full, update byte counters and compress
            if (ctx.c == 128) {
                ctx.t += ctx.c;
                compress(ctx, false);
                ctx.c = 0;
            }

            //Update temporary counter c
            uint256 c = ctx.c++;

            // b -> ctx.b
            uint256[4] memory b = ctx.b;
            uint8 a = uint8(input[i]);

            // ctx.b[c] = a
            assembly {
                mstore8(add(b, c), a)
            }
        }
    }

    function finalize(Blake2b_ctx memory ctx, uint64[8] memory out) internal view {
        // Add any uncounted bytes
        ctx.t += ctx.c;

        // Compress with finalization flag
        compress(ctx, true);

        //Flip little to big endian and store in output buffer
        for (uint256 i = 0; i < ctx.outlen / 8; i++) {
            out[i] = getWords(ctx.h[i]);
        }

        //Properly pad output if it doesn't fill a full word
        if (ctx.outlen < 64) {
            out[ctx.outlen / 8] = shift_right(getWords(ctx.h[ctx.outlen / 8]), 64 - 8 * (ctx.outlen % 8));
        }
    }

    //Helper function for full hash function
    function blake2b(
        bytes memory input,
        bytes memory key,
        bytes memory salt,
        bytes memory personalization,
        uint64 outlen
    ) public view returns (uint64[8] memory) {
        Blake2b_ctx memory ctx;
        uint64[8] memory out;

        init(ctx, outlen, key, formatInput(salt), formatInput(personalization));
        update(ctx, input);
        finalize(ctx, out);
        return out;
    }

    function blake2b(
        bytes memory input,
        bytes memory key,
        uint64 outlen
    ) public view returns (uint64[8] memory) {
        return blake2b(input, key, "", "", outlen);
    }

    // Utility functions

    //Flips endianness of words
    function getWords(uint64 a) public pure returns (uint64 b) {
        return
            ((a & MASK_0) / SHIFT_0) ^
            ((a & MASK_1) / SHIFT_1) ^
            ((a & MASK_2) / SHIFT_2) ^
            ((a & MASK_3) / SHIFT_3) ^
            ((a & MASK_4) * SHIFT_3) ^
            ((a & MASK_5) * SHIFT_2) ^
            ((a & MASK_6) * SHIFT_1) ^
            ((a & MASK_7) * SHIFT_0);
    }

    function shift_right(uint64 a, uint256 shift) public pure returns (uint64 b) {
        return uint64(a / 2**shift);
    }

    function shift_left(uint64 a, uint256 shift) public pure returns (uint64) {
        return uint64((a * 2**shift) % (2**64));
    }

    //bytes -> uint64[2]
    function formatInput(bytes memory input) public pure returns (uint64[2] memory output) {
        for (uint256 i = 0; i < input.length; i++) {
            output[i / 8] = output[i / 8] ^ shift_left(uint64(uint8(input[i])), 64 - 8 * ((i % 8) + 1));
        }
        output[0] = getWords(output[0]);
        output[1] = getWords(output[1]);
    }

    function formatOutput(uint64[8] memory input) public pure returns (bytes32[2] memory) {
        bytes32[2] memory result;

        for (uint256 i = 0; i < 8; i++) {
            result[i / 4] = result[i / 4] ^ bytes32(input[i] * 2**(64 * (3 - (i % 4))));
        }
        return result;
    }
}
