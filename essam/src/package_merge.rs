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

const INVALID_ID: u16 = std::u16::MAX;
const INVALID_PACKAGE_IDX: u16 = std::u16::MAX;

#[derive(Clone, Copy, Debug)]
struct CoinOrPackage {
    weight: u32,
    id: u16,
    package_idx: u16,
}

type Package = Vec<u16>;

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

    // Only keep track of the first 2(N - 1) coins.
    let max_num_coins = 2 * (num_symbols - 1);

    // The original set of coins.
    let original_coins = non_zero_order
        .iter()
        .map(|&idx| CoinOrPackage {
            weight: freqs[idx as usize],
            id: idx,
            package_idx: INVALID_PACKAGE_IDX,
        })
        .collect::<Vec<_>>();

    // Coins of the current denomination. We start with denomination 2^-L
    let mut current_coins = original_coins.clone();

    // The packages that was formed at the denomination being processed.
    let mut formed_packages = Vec::<CoinOrPackage>::new();

    // The data of the packaged coins. These encode which coins were merged to form the package.
    let mut packages_data = Vec::<Package>::new();

    // Process each denomination from the lowest value to the highest value.
    for _ in 0..max_length - 1 {
        formed_packages.clear();

        // Package every two consecutive coins.
        for half_coin_idx in 0..(current_coins.len() / 2) {
            let first_coin = &current_coins[half_coin_idx * 2];
            let second_coin = &current_coins[half_coin_idx * 2 + 1];

            match (first_coin, second_coin) {
                // If both are pure coins, then we create a new package for them.
                (
                    CoinOrPackage {
                        weight: first_weight,
                        id: first_id,
                        package_idx: INVALID_PACKAGE_IDX,
                    },
                    CoinOrPackage {
                        weight: second_weight,
                        id: second_id,
                        package_idx: INVALID_PACKAGE_IDX,
                    },
                ) => {
                    let mut package = Package::with_capacity(num_symbols);

                    package.push(*first_id);
                    package.push(*second_id);

                    packages_data.push(package);
                    formed_packages.push(CoinOrPackage {
                        weight: first_weight + second_weight,
                        id: INVALID_ID,
                        package_idx: (packages_data.len() - 1) as u16,
                    });
                }
                // If the first is a package, and the second is a coin.
                (
                    CoinOrPackage {
                        weight: first_weight,
                        id: INVALID_ID,
                        package_idx,
                    },
                    CoinOrPackage {
                        weight: second_weight,
                        id: second_id,
                        package_idx: INVALID_PACKAGE_IDX,
                    },
                ) => {
                    let package = &mut packages_data[*package_idx as usize];

                    package.push(*second_id);

                    formed_packages.push(CoinOrPackage {
                        weight: first_weight + second_weight,
                        id: INVALID_ID,
                        package_idx: *package_idx,
                    });
                }
                // If the second is a package, and the first is a coin.
                (
                    CoinOrPackage {
                        weight: first_weight,
                        id: first_id,
                        package_idx: INVALID_PACKAGE_IDX,
                    },
                    CoinOrPackage {
                        weight: second_weight,
                        id: INVALID_ID,
                        package_idx,
                    },
                ) => {
                    let package = &mut packages_data[*package_idx as usize];

                    package.push(*first_id);

                    formed_packages.push(CoinOrPackage {
                        weight: first_weight + second_weight,
                        id: INVALID_ID,
                        package_idx: *package_idx,
                    });
                }
                // If both are packages
                (
                    CoinOrPackage {
                        weight: first_weight,
                        id: INVALID_ID,
                        package_idx: first_package_idx,
                    },
                    CoinOrPackage {
                        weight: second_weight,
                        id: INVALID_ID,
                        package_idx: second_package_idx,
                    },
                ) => {
                    // This is not valid rust because I can't hold immutable and mutable references
                    // from the same vector.
                    //      let first_package = &mut packages_data[*first_package_idx as usize];
                    //      let second_package = &packages_data[*second_package_idx as usize];
                    //  So instead we do this hack, which destroys the package, but that's ok:
                    let mut first_package = Package::with_capacity(0);
                    std::mem::swap(
                        &mut packages_data[*first_package_idx as usize],
                        &mut first_package,
                    );
                    let second_package = &mut packages_data[*second_package_idx as usize];

                    second_package.extend(&first_package);

                    formed_packages.push(CoinOrPackage {
                        weight: first_weight + second_weight,
                        id: INVALID_ID,
                        package_idx: *second_package_idx,
                    });
                }
                _ => {
                    panic!("This should never happen!");
                }
            }
        }

        // Merge original_coins with formed_packages into next_coins
        current_coins.clear();

        let mut original_coins_idx = 0;
        let mut formed_packages_idx = 0;

        while original_coins_idx < original_coins.len()
            && formed_packages_idx < formed_packages.len()
            && current_coins.len() < max_num_coins
        {
            let original_coin = original_coins[original_coins_idx];
            let formed_package = formed_packages[formed_packages_idx];

            if original_coin.weight <= formed_package.weight {
                current_coins.push(original_coin);
                original_coins_idx += 1;
            } else {
                current_coins.push(formed_package);
                formed_packages_idx += 1;
            }
        }
        // Deal with left overs.
        while original_coins_idx < original_coins.len() && current_coins.len() < max_num_coins {
            current_coins.push(original_coins[original_coins_idx]);
            original_coins_idx += 1;
        }
        while formed_packages_idx < formed_packages.len() && current_coins.len() < max_num_coins {
            current_coins.push(formed_packages[formed_packages_idx]);
            formed_packages_idx += 1;
        }
    }

    // Now we compute the length of each symbol by seeing how many times each coin was picked.
    let mut lengths = vec![0; freqs.len()];

    for coin in current_coins {
        match coin {
            CoinOrPackage {
                weight: _,
                id,
                package_idx: INVALID_PACKAGE_IDX,
            } => {
                lengths[id as usize] += 1;
            }
            CoinOrPackage {
                weight: _,
                id: _,
                package_idx,
            } => {
                let package = &packages_data[package_idx as usize];
                for id in package.iter() {
                    lengths[*id as usize] += 1;
                }
            }
        }
    }

    Ok(lengths)
}
