use dynamodb_sheep_shed::DynamoDBSheepShed;
use sheep_shed::{Sheep, SheepShed, Weight};

use lambda_apigw_utils::prelude::*;

/// Create a Sieve of Eratostenes containing all the primes between 0 and n
fn sieve_of_eratosthenes(n: u64) -> Vec<u64> {
    assert!(n < usize::MAX as u64);
    if n < 2 {
        return vec![];
    }
    // Boolean array with a value for every number from 0 to n
    // Initially every number from 0 to n is considered prime
    // So the array is initialized at "true" for every index
    let mut tmp = vec![true; n as usize + 1];
    // 0 and 1 are not primes
    tmp[0] = false;
    tmp[1] = false;
    // Compute the square root of n, rounding up
    let sqrt_n = (n as f64).sqrt() as usize + 1;
    // Cast n to an usize instead of a u64
    let n_usize = n as usize;

    // For every candidate i from 2 to SquareRoot(n) rounded up excluded
    for i in 2..sqrt_n {
        // If the candidate i is prime
        // Exemple1: i = 2
        // Exemple2: i = 3
        if tmp[i] {
            // Then initialize j = i^2, this optimization work because of maths:
            // any multiple of our prime "i" that is inferior to i^2 MUST BE
            // a multiple of a previously processed prime, so already marked false.
            // When we process multiples of 3, we start at 9, skipping 6,
            // but 6 is 2*3 and was already taken care of when processing multiples of 2.
            // Exemple1: j = 4
            // Exemple2: j = 9
            let mut j = i * i;
            // As long as j is <= n
            while j <= n_usize {
                // Mark every j as "not prime"
                // Exemple1: 4, 6, 8, etc...
                // Exemple2: 9, 12, 15, 18, etc...
                tmp[j] = false;
                // Increment j by i
                // Exemple1: j += 2
                // Exemple2: j += 3
                j += i;
            }
        }
    }
    // At this point:
    // tmp[i] = true if i is prime
    // tmp[i] = false if i is NOT prime
    // Iterate over tmp to extract our sieve
    tmp.into_iter()
        // Enumerate provide the index alongside
        // the corresponding boolean value
        .enumerate()
        // We "filter" to keep only the prime indexes
        .filter(|(_index, is_prime)| *is_prime)
        // Indexes are of type usize but we want u64
        // So we "map" the values
        .map(|(index, _is_prime)| index as u64)
        // We collect
        .collect()
}

/// This wolf suffer from Obsessive-Compulsive disorder: it is hungry, but it cannot kill just any sheep !!
///
/// It is very important for the wolf that the [Weight] of the [Sheep] expressed in micro-grams is a
/// prime number!!! And of course, the bigest possible.
async fn wolf_ocd(_req: SimpleRequest<'_>) -> SimpleResult {
    let handle = tokio::runtime::Handle::current();

    // The wolf is multi-tasking: he knows retrieving infos on all the sheep
    // will take time, and computing primes too, so he is spawning a thread to
    // compute the primes.
    let sieve_max = (Weight::MAX.as_ug() as f64).sqrt() as u64;
    log::info!("spawning primes sieve generation (2 to {sieve_max})...");
    let f_sieve = handle.spawn_blocking(move || sieve_of_eratosthenes(sieve_max));

    // Then another thread to retrieve the sheeps
    log::info!("retrieving all the sheeps...");
    let f_sheeps = handle.spawn_blocking(move || {
        log::info!("create a shed instance");
        DynamoDBSheepShed::new(dynamo())
            .sheep_iter()
            .map(|iter| iter.collect::<Vec<_>>())
    });

    // Wait both thread finishes
    let sieve = f_sieve.await.unwrap();
    log::info!("sieve contains {} primes", sieve.len());

    let sheeps = f_sheeps.await.unwrap()?;
    log::info!("sheep list contains {} sheep", sheeps.len());

    // Find a suitable sheep
    let sheep_to_eat = sheeps
        .into_iter()
        .filter(|sheep| {
            let sheep_weight_ug = sheep.weight.as_ug();
            for &prime in &sieve {
                if sheep_weight_ug % prime == 0 {
                    return false;
                }
            }
            true
        })
        .fold(None, |heaviest_sheep, current_sheep| {
            // If there is no heaviest_sheep or heaviest_sheep is lighter than the current sheep
            if !heaviest_sheep
                .as_ref()
                .is_some_and(|hs: &Sheep| hs.weight > current_sheep.weight)
            {
                Some(current_sheep)
            } else {
                heaviest_sheep
            }
        });

    // If we found a suitable sheep, eat it and return 204
    if let Some(sheep) = &sheep_to_eat {
        let sheep_tattoo = sheep.tattoo.clone();
        log::info!("wolf will eat {sheep}");
        let _ = handle
            .spawn_blocking(move || {
                log::info!("create a shed instance");
                DynamoDBSheepShed::new(dynamo()).kill_sheep(&sheep_tattoo)
            })
            .await
            .unwrap()
            .map_err(|e| {
                // In this specific case, we consider SheepNotPresent to be a 500
                if let sheep_shed::errors::Error::SheepNotPresent(_) = e {
                    SimpleError::Custom {
                        code: 500,
                        message: e.to_string(),
                    }
                } else {
                    // Any other error will follow the standard conversion
                    e.into()
                }
            })?;
        simple_response!(204)
    // Else do nothing and return 404
    } else {
        log::info!("it seems the wolf will continue to starve...");
        simple_response!(404, json!({"message": "No fitting sheep"}))
    }
}

lambda_main!(async wolf_ocd, dynamo = aws_sdk_dynamodb::Client);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn small_sieve_of_eratosthenes() {
        let soe = sieve_of_eratosthenes(1000);
        let primes_to_1000 = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179,
            181, 191, 193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271,
            277, 281, 283, 293, 307, 311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379,
            383, 389, 397, 401, 409, 419, 421, 431, 433, 439, 443, 449, 457, 461, 463, 467, 479,
            487, 491, 499, 503, 509, 521, 523, 541, 547, 557, 563, 569, 571, 577, 587, 593, 599,
            601, 607, 613, 617, 619, 631, 641, 643, 647, 653, 659, 661, 673, 677, 683, 691, 701,
            709, 719, 727, 733, 739, 743, 751, 757, 761, 769, 773, 787, 797, 809, 811, 821, 823,
            827, 829, 839, 853, 857, 859, 863, 877, 881, 883, 887, 907, 911, 919, 929, 937, 941,
            947, 953, 967, 971, 977, 983, 991, 997,
        ];
        assert_eq!(soe, primes_to_1000);
    }

    #[test]
    fn is_prime_101() {
        let soe = sieve_of_eratosthenes(101);
        assert_eq!(soe.last().cloned().unwrap(), 101);
    }

    #[test]
    fn sieve_0() {
        let soe = sieve_of_eratosthenes(0);
        assert_eq!(soe, Vec::<u64>::new());
    }

    #[test]
    fn sieve_1() {
        let soe = sieve_of_eratosthenes(1);
        assert_eq!(soe, Vec::<u64>::new());
    }

    #[test]
    fn sieve_2() {
        let soe = sieve_of_eratosthenes(2);
        assert_eq!(soe, vec![2]);
    }

    #[test]
    fn is_not_prime_102() {
        let soe = sieve_of_eratosthenes(102);
        assert_eq!(soe.last().cloned().unwrap(), 101);
    }
}
