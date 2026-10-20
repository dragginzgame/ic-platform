use ic_platform::Principal;
use std::{hint::black_box, time::Instant};

fn main() {
    let rounds: usize = 500_000;
    for length in [10, 29] {
        let principal = Principal::from_slice(&vec![42; length]);
        let text = principal.to_text();
        assert_eq!(Principal::from_text(&text).unwrap(), principal);
        for operation in ["parse", "format"] {
            // Warm allocator and instruction paths outside measurement.
            for _ in 0..10_000 {
                black_box(Principal::from_text(black_box(&text)).unwrap());
                black_box(black_box(principal).to_text());
            }
            let start = Instant::now();
            if operation == "parse" {
                for _ in 0..rounds {
                    black_box(Principal::from_text(black_box(&text)).unwrap());
                }
            } else {
                for _ in 0..rounds {
                    black_box(black_box(principal).to_text());
                }
            }
            println!(
                "{length}\t{operation}\t{rounds}\t{}",
                start.elapsed().as_nanos()
            );
        }
    }
}
