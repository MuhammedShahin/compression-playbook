// Implements the package-merge algorithm.
//
// Reference: A Fast Algorithm for Optimal Length-Limited Huffman Codes by Larmore/Hirschberg
//
// The Coin Collector Problem
// ===========================
// This algorithm maps the problem of building a length-limited huffman tree to the "Coin
// Collector's Problem". In this problem, we have different coins of different denominations,
// and different numismatic value. So if we go to a bank, all of the coins having the same
// denomination are equivalent. But to us, some of these coins are more valuable than others.
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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageMergeError {
    #[error("invalid requested max length")]
    InvalidMaxLength,
}

#[derive(Default, Clone, Copy, Debug)]
struct Coin {
    weight: u32,
    id: u16,
}

#[derive(Default)]
struct Package {
    weight: u32,
    coins: Vec<u16>,
}

#[derive(Copy, Clone, Default)]
struct PureCoinOrPackageIdx {
    idx: usize,
    is_pure_coin: bool,
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
    let least_allowable_max_length = {
        let max_code = num_symbols - 1;
        let num_bits = std::mem::size_of_val(&max_code) * 8; // = 64

        num_bits - max_code.leading_zeros() as usize
    };

    if max_length < least_allowable_max_length {
        return Err(PackageMergeError::InvalidMaxLength);
    } else if max_length == least_allowable_max_length {
        // If the required max length is the least allowable max length, then the only solution is
        // to have all symbols with the same length.
        return Ok(freqs
            .iter()
            .map(|&freq| {
                if freq != 0 {
                    least_allowable_max_length as u8
                } else {
                    0 as u8
                }
            })
            .collect());
    }

    // The original set of coins.
    let pure_coins = non_zero_order
        .iter()
        .map(|&idx| Coin {
            weight: freqs[idx as usize],
            id: idx,
        })
        .collect::<Vec<_>>();

    // The packages that was formed at the previous denomination.
    let mut formed_packages_prev = Vec::<Package>::new();
    formed_packages_prev.resize_with(num_symbols, || Package {
        weight: 0,
        coins: Vec::with_capacity(max_length),
    });
    let mut formed_packages_prev_len = 0;

    // The packages that will be formed at the current denomination.
    let mut formed_packages_cur = Vec::<Package>::new();
    formed_packages_cur.resize_with(num_symbols, || Package {
        weight: 0,
        coins: Vec::with_capacity(max_length),
    });

    // Process each denomination from the lowest value to the highest value.
    for _ in 0..max_length - 1 {
        let mut formed_packages_cur_len = 0;

        let mut coins_to_merge = [
            PureCoinOrPackageIdx::default(),
            PureCoinOrPackageIdx::default(),
        ];
        let mut is_first_coin = true;

        let mut form_package = |idx: usize,
                                is_pure_coin: bool,
                                pure_coins: &[Coin],
                                formed_packages_prev: &mut [Package]|
         -> () {
            let coins_to_merge_idx = if is_first_coin { 0 } else { 1 };
            is_first_coin = !is_first_coin;

            coins_to_merge[coins_to_merge_idx] = PureCoinOrPackageIdx { idx, is_pure_coin };

            if coins_to_merge_idx == 0 {
                return;
            }

            let first_coin_idx;
            let second_coin_idx;

            // Always make the package first
            if coins_to_merge[0].is_pure_coin && !coins_to_merge[1].is_pure_coin {
                first_coin_idx = coins_to_merge[1];
                second_coin_idx = coins_to_merge[0];
            } else {
                first_coin_idx = coins_to_merge[0];
                second_coin_idx = coins_to_merge[1];
            }

            match (first_coin_idx, second_coin_idx) {
                // If both are pure coins, then we create a new package for them.
                (
                    PureCoinOrPackageIdx {
                        idx: idx1,
                        is_pure_coin: true,
                    },
                    PureCoinOrPackageIdx {
                        idx: idx2,
                        is_pure_coin: true,
                    },
                ) => {
                    let coin1 = &pure_coins[idx1];
                    let coin2 = &pure_coins[idx2];
                    let package = &mut formed_packages_cur[formed_packages_cur_len];

                    package.weight = coin1.weight + coin2.weight;
                    package.coins.resize(2, 0);
                    package.coins[0] = idx1 as u16;
                    package.coins[1] = idx2 as u16;

                    formed_packages_cur_len += 1;
                }
                // If the first is a package, and the second is a coin.
                (
                    PureCoinOrPackageIdx {
                        idx: package_idx,
                        is_pure_coin: false,
                    },
                    PureCoinOrPackageIdx {
                        idx: coin_idx,
                        is_pure_coin: true,
                    },
                ) => {
                    let coin = &pure_coins[coin_idx];

                    std::mem::swap(
                        &mut formed_packages_prev[package_idx],
                        &mut formed_packages_cur[formed_packages_cur_len],
                    );

                    let package = &mut formed_packages_cur[formed_packages_cur_len];
                    package.weight += coin.weight;
                    package.coins.push(coin_idx as u16);

                    formed_packages_cur_len += 1;
                }
                // If both are packages
                (
                    PureCoinOrPackageIdx {
                        idx: idx1,
                        is_pure_coin: false,
                    },
                    PureCoinOrPackageIdx {
                        idx: idx2,
                        is_pure_coin: false,
                    },
                ) => {
                    std::mem::swap(
                        &mut formed_packages_prev[idx1],
                        &mut formed_packages_cur[formed_packages_cur_len],
                    );

                    let first_package = &mut formed_packages_cur[formed_packages_cur_len];
                    let second_package = &formed_packages_prev[idx2];

                    first_package.coins.extend(&second_package.coins);
                    first_package.weight += second_package.weight;

                    formed_packages_cur_len += 1;
                }

                _ => {
                    panic!("This should never happen!");
                }
            }
        };

        merge(
            &pure_coins,
            &mut formed_packages_prev[..formed_packages_prev_len],
            &mut form_package,
        );

        std::mem::swap(&mut formed_packages_prev, &mut formed_packages_cur);
        formed_packages_prev_len = formed_packages_cur_len;
    }

    // Now we compute the length of each symbol by seeing how many times each coin was picked.
    let mut lengths = vec![0; freqs.len()];
    for coin in &pure_coins {
        lengths[coin.id as usize] += 1;
    }

    // Only keep track of the first 2(N - 1) coins, out of which we have N pure coins, which mean
    // that the number of required packages are N - 2.
    let max_packages = num_symbols - 2;

    for package in &formed_packages_prev[..max_packages] {
        for coin_idx in &package.coins {
            lengths[pure_coins[*coin_idx as usize].id as usize] += 1;
        }
    }

    Ok(lengths)
}

fn merge(
    coins: &[Coin],
    packages: &mut [Package],
    op: &mut impl FnMut(usize, bool, &[Coin], &mut [Package]),
) {
    let mut coins_idx = 0;
    let mut packages_idx = 0;

    while coins_idx < coins.len() && packages_idx < packages.len() {
        let first = &coins[coins_idx];
        let second = &packages[packages_idx];

        if first.weight <= second.weight {
            op(coins_idx, true, coins, packages);
            coins_idx += 1;
        } else {
            op(packages_idx, false, coins, packages);
            packages_idx += 1;
        }
    }
    // Deal with left overs.
    for idx in coins_idx..coins.len() {
        op(idx, true, coins, packages);
    }
    for idx in packages_idx..packages.len() {
        op(idx, false, coins, packages);
    }
}
