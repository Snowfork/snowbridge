use crate::header::Header;
use ethereum_types::U256;
use sp_std::convert::TryFrom;

const DIFFICULTY_BOUND_DIVISOR: u32 = 11;  // right-shifts equivalent to division by 2048
const EXP_DIFFICULTY_PERIOD: u32 = 100000;
const MINIMUM_DIFFICULTY: u32 = 131072;

pub enum BombDelay {
    // Specification EIP-649: https://eips.ethereum.org/EIPS/eip-649
    Byzantium = 3000000,
    // Specification EIP-1234: https://eips.ethereum.org/EIPS/eip-1234
    Constantinople = 5000000,
    // // Specification EIP-2384: https://eips.ethereum.org/EIPS/eip-2384
    MuirGlacier = 9000000,
}

/// This difficulty calculation follows Byzantium rules (https://eips.ethereum.org/EIPS/eip-649)
/// and shouldn't be used to calculate difficulty prior to the Byzantium fork.
fn calc_difficulty(
    bomb_delay: BombDelay,
    time: u64,
    parent: &Header,
) -> Result<U256, &'static str> {
    let bomb_delay_from_parent = bomb_delay as u32 - 1;

    let block_time = time.checked_sub(parent.timestamp).ok_or("Invalid block time")?;
    let block_time_div_9 = i32::try_from(block_time / 9).or(Err("Invalid block time"))?;
    let sigma2 = match parent.has_ommers() {
        true => 2 - block_time_div_9,
        false => 1 - block_time_div_9,
    }.max(-99);

    let mut difficulty_without_exp = parent.difficulty;
    if sigma2 < 0 {
        difficulty_without_exp -= parent.difficulty >> DIFFICULTY_BOUND_DIVISOR * (sigma2 * -1) as u32;
    } else {
        difficulty_without_exp += parent.difficulty >> DIFFICULTY_BOUND_DIVISOR * sigma2 as u32;
    }

    // Technically, the Yellow Paper applies the minimum difficulty after adding exp, but
    // Geth does this
    difficulty_without_exp = difficulty_without_exp.max(MINIMUM_DIFFICULTY.into()); // TODO parameterize

    let fake_block_number: u32 = u32::try_from(parent.number)
        .or(Err("Invalid parent number"))?
        .saturating_sub(bomb_delay_from_parent);
    let period_count = fake_block_number / EXP_DIFFICULTY_PERIOD;

    // If period_count < 2, exp is fractional and we can skip adding it
    if period_count >= 2 {
        return Ok(difficulty_without_exp + U256::from(2).pow((period_count - 2).into()));
    }

    Ok(difficulty_without_exp)
}


// Correct block numbers for mainnet and various testnets can be found here:
// https://github.com/ethereum/go-ethereum/blob/498458b4102c0d32d7453035a115e6b9df5e485d/params/config.go#L55-L258
// Specification EIP-649: https://eips.ethereum.org/EIPS/eip-649
const BYZANTIUM_FORK_BLOCK_MAINNET: u32 = 4370000;
const BYZANTIUM_FORK_BLOCK_ROPSTEN: u32 = 1700000;
// Specification EIP-1234: https://eips.ethereum.org/EIPS/eip-1234
const CONSTANTINOPLE_FORK_BLOCK_MAINNET: u32 = 7280000;
const CONSTANTINOPLE_FORK_BLOCK_ROPSTEN: u32 = 4230000;
// Specification EIP-2384: https://eips.ethereum.org/EIPS/eip-2384
const MUIR_GLACIER_FORK_BLOCK_MAINNET: u32 = 9200000;
const MUIR_GLACIER_FORK_BLOCK_ROPSTEN: u32 = 7117117;
