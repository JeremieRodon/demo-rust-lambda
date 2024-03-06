use aws_sdk_dynamodb::Client;
use dynamodb_sheep_shed::DynamoDBSheepShed;
use sheep_shed::{Sheep, SheepShed, Weight};

use lambda_apigw_commons::prelude::*;

/// Create a Sieve of Eratostenes containing all primes from 2 to n
fn sieve_of_eratosthenes(n: u64) -> Vec<u64> {
    assert!(n < usize::MAX as u64);
    // Boolean array with a value for every number from 0 to n
    let mut tmp = vec![true; n as usize + 1];
    // 0 and 1 are not primes
    tmp[0] = false;
    tmp[1] = false;
    let sqrt_n = (n as f64).sqrt() as usize + 1;
    let n_usize = n as usize;
    for i in 2..sqrt_n {
        if tmp[i] {
            let mut j = i * i;
            while j <= n_usize {
                tmp[j] = false;
                j += i;
            }
        }
    }
    tmp.into_iter()
        .enumerate()
        .filter_map(|(i, b)| if b { Some(i as u64) } else { None })
        .collect()
}

/// This wolf suffer from Obsessive-Compulsive disorder: it is hungry, but it cannot kill just any sheep !!
///
/// It is very important for the wolf that the [Weight] of the [Sheep] expressed in micro-grams is a
/// prime number!!! And of course, the bigest possible.
async fn wolf_ocd(_req: SimpleRequest<'_>) -> SimpleResult {
    let handle = tokio::runtime::Handle::current();

    // The wolf is multi-tasking: he knows retrieving infos on all the sheep
    // will take time, and computing primes too
    let sieve_max = (Weight::MAX.as_ug() as f64).sqrt() as u64;
    log::info!("spawning primes sieve generation (2 to {sieve_max})...");
    let f_sieve = handle.spawn_blocking(move || sieve_of_eratosthenes(sieve_max));

    log::info!("retrieving all the sheeps...");
    let f_sheeps = handle.spawn_blocking(move || {
        log::info!("create a shed instance");
        DynamoDBSheepShed::new(dynamo())
            .sheep_iter()
            .map(|iter| iter.collect::<Vec<_>>())
    });

    let sieve = f_sieve.await.unwrap();
    log::info!("sieve contains {} primes", sieve.len());

    let sheeps = f_sheeps.await.unwrap()?;
    log::info!("sheep list contains {} sheep", sheeps.len());

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

    if let Some(sheep) = &sheep_to_eat {
        let sheep_tatoo = sheep.tatoo.clone();
        log::info!("wolf will eat {sheep}");
        let _ = handle
            .spawn_blocking(move || {
                log::info!("create a shed instance");
                DynamoDBSheepShed::new(dynamo()).kill_sheep(&sheep_tatoo)
            })
            .await
            .unwrap()?;
        simple_response!(204)
    } else {
        log::info!("it seems the wolf will continue to starve...");
        simple_response!(404, json!({"message": "No fitting sheep"}))
    }
}

lambda_main!(wolf_ocd, dynamo(DYNAMO) = Client);

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
    fn is_not_prime_102() {
        let soe = sieve_of_eratosthenes(102);
        assert_eq!(soe.last().cloned().unwrap(), 101);
    }
}
