use lambda_apigw_utils::prelude::*;

fn ackermann_2ary_iter(m: u128, n: u128) -> u128 {
    let mut stack = vec![m, n];
    loop {
        // match (n, m)
        match (/* Pop n */ stack.pop(), /* Pop m */ stack.pop()) {
            // If the second pop is a None, then the first one
            // was the last element of the stack, we are finished
            (Some(result), None) => return result,
            // If m == 0
            // r1: A(0, n) => n + 1
            (Some(n), Some(m)) if m == 0 => stack.push(n + 1),
            // If n == 0
            // r2: A(m + 1, 0) => A(m, 1)
            // r2: A(m, 0) => A(m - 1, 1)
            (Some(n), Some(m)) if n == 0 => {
                // Push m first
                stack.push(m - 1);
                // Push n
                stack.push(1);
            }
            // Else
            // r3: A(m + 1, n + 1) => A(m, A(m + 1, n))
            // r3: A(m, n) => A(m - 1, A(m, n - 1))
            (Some(n), Some(m)) => {
                // Push m - 1
                stack.push(m - 1);
                stack.push(m);
                stack.push(n - 1);
            }
            (None, None) | (None, Some(_)) => {
                unreachable!("we always return a result before those situations")
            }
        }
    }
}

async fn run_ackermann(req: SimpleRequest<'_>) -> SimpleResult {
    let parameters = req.parameters;
    let m = parameters.get("m").unwrap().parse().unwrap();
    let n = parameters.get("n").unwrap().parse().unwrap();
    log::info!("Running A({m}, {n})...");
    let result = tokio::task::spawn_blocking(move || ackermann_2ary_iter(m, n))
        .await
        .unwrap();
    log::info!("A({m}, {n}) = {result}");

    simple_response!(200, json!({"result": result}))
}

lambda_main!(async run_ackermann);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn ackermann_2ary_iter_r1() {
        let ns = vec![0, 1, 13, 23897];
        // A(0, n) => n + 1
        for n in ns {
            assert_eq!(ackermann_2ary_iter(0, n), n + 1);
        }
    }

    #[test]
    fn ackermann_2ary_iter_r2() {
        let ms = vec![0, 1, 2, 3];
        // A(m + 1, 0) => A(m, 1)
        for m in ms {
            assert_eq!(ackermann_2ary_iter(m + 1, 0), ackermann_2ary_iter(m, 1));
        }
    }

    #[test]
    fn ackermann_2ary_iter_r3() {
        let (m, n) = (1, 1);
        // A(m + 1, n + 1) => A(m, A(m + 1, n))
        assert_eq!(
            ackermann_2ary_iter(m + 1, n + 1),
            ackermann_2ary_iter(m, ackermann_2ary_iter(m + 1, n))
        );
    }

    #[test]
    fn ackermann_2ary_iter_known_res() {
        // A(1, 2) => 4
        assert_eq!(ackermann_2ary_iter(1, 2), 4);
    }

    #[test]
    #[ignore = "this is more a benchmark and it takes ages"]
    fn fake_test() {
        use std::time;

        // A(2, 10000) = 20003
        // 3 seconds
        let start = time::Instant::now();
        let res = ackermann_2ary_iter(2, 10_000);
        let elapsed = time::Instant::now() - start;
        println!("ackermann_2ary_iter(2, 10_000) = {res}");
        println!("Elapsed: {elapsed:?}");
        println!();

        // A(2, 50000) = 100003
        // iterations=5000350006
        // max_stack_size=100003
        // 97 seconds
        let start = time::Instant::now();
        let res = ackermann_2ary_iter(2, 50_000);
        let elapsed = time::Instant::now() - start;
        println!("ackermann_2ary_iter(2, 50_000) = {res}");
        println!("Elapsed: {elapsed:?}");
        println!();

        // MAX A(3, 14)
        // A(3, 14) = 131069
        // iterations=11452590818
        // max_stack_size=131069
        let start = time::Instant::now();
        let res = ackermann_2ary_iter(3, 14);
        let elapsed = time::Instant::now() - start;
        println!("ackermann_2ary_iter(3, 14) = {res}");
        println!("Elapsed: {elapsed:?}");
        println!();

        // MAX A(4, 1)
        // A(4, 1) = 65533
        // iterations=2862984011
        // max_stack_size=65533
        let start = time::Instant::now();
        let res = ackermann_2ary_iter(4, 1);
        let elapsed = time::Instant::now() - start;
        println!("ackermann_2ary_iter(4, 1) = {res}");
        println!("Elapsed: {elapsed:?}");
        println!();
    }
}
