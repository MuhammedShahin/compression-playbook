// Implements the package-merge algorithm.
//
// Reference: A Fast Algorithm for Optimal Length-Limited Huffman Codes by Larmore/Hirschberg
// Follows the implementation from: https://create.stephan-brumme.com/length-limited-prefix-codes/
//
// The Coin Collector Problem
// ===========================
// This algorithm maps the problem of building a length-limited huffman tree to the "Coin
// Collector's Problem". In this problem, we have different coins of different denominations,
// and different numismatic value (weights). So if we go to a bank, all of the coins having the
// same denomination are equivalent. But to us, some of these coins are more valuable than others.
//
// Suppose we want to buy something of cost X. Then the problem is to find the set of coins with
// the least numismatic value that allows us to do as such.
//
// Assumptions
// ===========
// 1. The denominations are powers of 2. This means that each coin has a value of 2^k for some k
//    that can be negative.
// 2. The cost X has to be representable as the sum of powers of 2. Otherwise, no set of coins has
//    exactly sum value of X. These are called dyadic numbers.
//
// The Package Merge Algorithm
// ============================
// 1. Group the coins by their denominations.
// 2. Let X' represent the price we still have to pay, and initialize it to X.
// 2. Let d be the denomination, and starting from the lowest denominations to the highest, we have
//    the following cases:
//        a. When the current cost X' is represented as a sum of powers of 2, is the current
//           denomination one of the terms? If that's the case, then update the price we have to
//           pay X' := X' - 2^d, and remove this coin.
//        b. Else:
//               I. we "package" the coins of this denomination, going from the lowest weight to
//                  the heighest weight, such that each two consecutive coins will be treated as a
//                  single coin of the next denomination (2^d + 2^d = 2^(d+1)), and whose weight is
//                  the sum of the original coins weights. If there remains a single coin (which
//                  will have the highest weight), then discard it.
//              II. Merge the packaged coins into the group of the coins of the next denomination.
//
// Mapping to Length-Limited Huffman Trees
// ========================================
// 1. We want to buy an item whose cost is N - 1.
// 2. Coin denominations represent code lengths (2^-l).
// 3. Numismatic values are the code frequencies/probabilities.
// 4. The coins themselves will be the symbols, but with each symbol repeated as a coin for each
//    allowable code length. This menas we will have N*L coins.
// 5. The length of each symbol will be the number of times we picked the symbol across all
//    denominations.
//
// Remarks
// =======
// 1. Since we have to buy an item with integer value, we will have to keep packaging the coins until
//    we get them to heighest denomination 2^-1. We will then choose the first 2(N - 1) coins with
//    least numismatic values so that we can buy our item of cost N - 1.
// 2. At any point in the packaging/merging process, if we get a group of coins/packages with size
//    greater than 2(N - 1), then we can safely discard the coins with the highest numismatic
//    values to bring the size back to 2(N - 1).
//
// Proof
// =====
// Read the paper!

use crate::bitset::Bitset;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageMergeError {
    #[error("invalid requested max length")]
    InvalidMaxLength,
}

pub fn package_merge(freqs: &[u32], max_length: usize) -> Result<Vec<u8>, PackageMergeError> {
    // Handle trivial cases with having only one or two symbols.
    if freqs.len() == 1 {
        return Ok([if freqs[0] > 0 { 1 } else { 0 }].to_vec());
    }

    if freqs.len() == 2 {
        return Ok([
            if freqs[0] > 0 { 1 } else { 0 },
            if freqs[1] > 0 { 1 } else { 0 },
        ]
        .to_vec());
    }

    // First we sort frequencies in an ascending order, and get rid of symbols with 0 frequency
    let mut order = (0..freqs.len() as u16).collect::<Vec<u16>>();
    order.sort_unstable_by_key(|&idx1| freqs[idx1 as usize]);

    let non_zero_order: &[u16];

    if let Some(first_non_zero) = order.iter().position(|&idx| freqs[idx as usize] != 0) {
        non_zero_order = &order[first_non_zero as usize..order.len()];
    } else {
        return Ok(vec![0; freqs.len()]);
    }

    let num_symbols = non_zero_order.len();

    // Handle trivial cases with having only one or two symbols.
    if num_symbols <= 2 {
        return Ok(freqs
            .iter()
            .map(|&freq| if freq > 0 { 1 } else { 0 })
            .collect());
    }

    // Check if the requested max_length is possible
    if 1 << max_length < freqs.len() {
        return Err(PackageMergeError::InvalidMaxLength);
    } else if 1 << max_length == freqs.len() {
        return Ok(freqs
            .iter()
            .map(|&freq| if freq != 0 { max_length as u8 } else { 0 as u8 })
            .collect());
    }

    // The original set of coins.
    let pure_coins = non_zero_order
        .iter()
        .map(|&idx| freqs[idx as usize])
        .collect::<Vec<_>>();

    let mut prev_coins = pure_coins.clone();
    let mut cur_coins = Vec::with_capacity(2 * num_symbols);

    // Allocate bit mask for each level of size 2 * num_symbols.
    // At level "l" the bit at position "i" represent whether the coin at this location is a
    // package or not.
    let mut merged_mask = Bitset::with_capacity(2 * num_symbols * max_length);

    // Process each denomination from the lowest value to the highest value.
    let mut denom = 0;
    while denom < max_length - 1 {
        cur_coins.clear();
        cur_coins.push(prev_coins[0]);
        cur_coins.push(prev_coins[1]);

        let prev_coins_even_len = prev_coins.len() & !1;
        let mut cur_package_weight = prev_coins[0] + prev_coins[1];
        let mut cur_package_idx = 0;
        let mut pure_coins_idx = 2;

        let mut bit_idx = 2 * num_symbols * denom + 2;

        while pure_coins_idx < pure_coins.len() {
            if pure_coins[pure_coins_idx] < cur_package_weight {
                cur_coins.push(pure_coins[pure_coins_idx]);
                pure_coins_idx += 1;
                bit_idx += 1;
            } else {
                cur_coins.push(cur_package_weight);
                merged_mask.insert(bit_idx);
                bit_idx += 1;

                cur_package_idx += 1;
                if cur_package_idx * 2 >= prev_coins_even_len {
                    break;
                }
                cur_package_weight =
                    prev_coins[2 * cur_package_idx] + prev_coins[2 * cur_package_idx + 1];
            }
        }
        // Add remaining coins
        while pure_coins_idx < pure_coins.len() {
            cur_coins.push(pure_coins[pure_coins_idx]);
            pure_coins_idx += 1;
            bit_idx += 1;
        }
        // Add remaining packages
        loop {
            cur_coins.push(cur_package_weight);
            merged_mask.insert(bit_idx);

            cur_package_idx += 1;
            bit_idx += 1;

            if cur_package_idx * 2 >= prev_coins_even_len {
                break;
            }
            cur_package_weight =
                prev_coins[2 * cur_package_idx] + prev_coins[2 * cur_package_idx + 1];
        }

        std::mem::swap(&mut cur_coins, &mut prev_coins);
        denom += 1;

        if cur_coins == prev_coins {
            break;
        }
    }

    // Using only the merged_mask we can deduce which coins were packaged at each level, and that
    // contributed to the final result. This relies on the following observations:
    // 1. The pure coins always maintain their relative order at each denomination after packaging/merging.
    // 2. The coins that contribute to the packaging/merging at specific denomination are the first
    //    2*num_merged coins of the denomination before it.
    //
    // By starting at the highest denomination, we can count the merged packages and trace back to
    // the first 2*num_merged coins of the previous denomination to find the pure coins involved.
    // This helps us infer the contributing coins of the previous denomination.
    //
    // The length of a symbol shows is the number of times its pure coin was part of the solution.

    let mut sorted_lengths = vec![0; num_symbols];
    let mut num_relevant_coins = 2 * (num_symbols - 1);

    for denom in (0..denom).rev() {
        let bit_from = 2 * num_symbols * denom;
        let bit_to = bit_from + num_relevant_coins;

        let num_merged = merged_mask.count_ones_sliced(bit_from, bit_to);
        let num_not_merged = (bit_to - bit_from) - num_merged;

        for length in sorted_lengths[0..num_not_merged].iter_mut() {
            *length += 1;
        }

        num_relevant_coins = num_merged * 2;
    }

    // The smallest denomination has no packages.
    for length in sorted_lengths[0..num_relevant_coins].iter_mut() {
        *length += 1;
    }

    // Return the original order.
    let mut lengths = vec![0; freqs.len()];
    for idx in 0..sorted_lengths.len() {
        lengths[non_zero_order[idx] as usize] = sorted_lengths[idx];
    }

    Ok(lengths)
}
