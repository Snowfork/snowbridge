// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

/// @dev A second, independent model of `Bitfield.subsample`, written from the sampling rule:
/// draw `keccak256(seed, i) mod size` for i = 0, 1, 2, ..., keep a draw when that validator
/// claimed to sign and is not already kept, and stop after `n` kept draws.
///
/// It shares no code or structure with the library and the pre-assembly loop it is checked
/// against:
///  - claims and the sample are sorted index lists, not bitfields; membership is a search;
///  - the draw hashes `abi.encodePacked(seed, i)` rather than writing scratch memory;
///  - the parameter checks are its own, and report `ok = false` instead of reverting;
///  - the output bitfield is built only at the end, from the kept indices.
/// Arithmetic is checked, so an overflow panics instead of wrapping.
library SpecSubsample {
    function subsample(uint256 seed, uint256[] memory prior, uint256 size, uint256 n)
        internal
        pure
        returns (bool ok, uint256[] memory out)
    {
        // One word per 256 validators, rounded up, and enough claims to fill the sample.
        if (prior.length != (size + 255) / 256) {
            return (false, out);
        }
        uint256[] memory claimed = claimedBelow(prior, size);
        if (n > claimed.length) {
            return (false, out);
        }

        uint256[] memory kept = new uint256[](n);
        uint256 count;
        for (uint256 i = 0; count < n; i++) {
            uint256 draw = uint256(keccak256(abi.encodePacked(seed, i))) % size;
            if (contains(claimed, claimed.length, draw) && !containsUnsorted(kept, count, draw)) {
                kept[count++] = draw;
            }
        }

        out = new uint256[](prior.length);
        for (uint256 k = 0; k < n; k++) {
            out[kept[k] / 256] += 2 ** (kept[k] % 256);
        }
        return (true, out);
    }

    /// Ascending indices below `size` whose bit is set in `prior`.
    function claimedBelow(uint256[] memory prior, uint256 size)
        internal
        pure
        returns (uint256[] memory list)
    {
        list = new uint256[](size);
        uint256 m;
        for (uint256 v = 0; v < size; v++) {
            if ((prior[v / 256] / 2 ** (v % 256)) % 2 == 1) {
                list[m++] = v;
            }
        }
        assembly {
            mstore(list, m)
        }
    }

    /// Binary search in the first `len` entries of an ascending list.
    function contains(uint256[] memory sorted, uint256 len, uint256 x)
        internal
        pure
        returns (bool)
    {
        uint256 lo = 0;
        uint256 hi = len;
        while (lo < hi) {
            uint256 mid = (lo + hi) / 2;
            if (sorted[mid] == x) return true;
            if (sorted[mid] < x) lo = mid + 1;
            else hi = mid;
        }
        return false;
    }

    /// Linear search in the first `len` entries.
    function containsUnsorted(uint256[] memory list, uint256 len, uint256 x)
        internal
        pure
        returns (bool)
    {
        for (uint256 k = 0; k < len; k++) {
            if (list[k] == x) return true;
        }
        return false;
    }
}
