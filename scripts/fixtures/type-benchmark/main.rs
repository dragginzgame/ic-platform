use ic_platform::{
    Account, Principal, Subaccount, account::ExternalAccount, principal::ExternalPrincipal,
};
use std::{hint::black_box, mem::size_of, time::Instant};

fn measure(name: &str, mut operation: impl FnMut()) {
    for _ in 0..10_000 {
        operation();
    }
    let start = Instant::now();
    for _ in 0..500_000 {
        operation();
    }
    println!("{name}\t{}", start.elapsed().as_nanos() as f64 / 500_000.0);
}

fn main() {
    println!("size_owned_principal\t{}", size_of::<Principal>());
    println!(
        "size_external_principal\t{}",
        size_of::<ExternalPrincipal>()
    );
    println!("size_owned_account\t{}", size_of::<Account>());
    println!("size_external_account\t{}", size_of::<ExternalAccount>());
    println!("size_subaccount\t{}", size_of::<Subaccount>());
    for length in [10, 29] {
        for subaccount in [None, Some([0; 32]), Some([42; 32])] {
            let owned = Account {
                owner: Principal::from_slice(&[42; 29][..length]),
                subaccount,
            };
            let external: ExternalAccount = owned.into();
            let text = owned.to_string();
            assert_eq!(text, external.to_string());
            let label = format!(
                "{length}-{}",
                if subaccount.is_none() {
                    "none"
                } else if subaccount == Some([0; 32]) {
                    "zero"
                } else {
                    "named"
                }
            );
            for run in 0..5 {
                // Alternate implementation order to reduce systematic drift.
                for candidate in if run % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    let implementation = if candidate { "owned" } else { "external" };
                    measure(&format!("{label}-{run}-{implementation}-parse"), || {
                        if candidate {
                            black_box(black_box(&text).parse::<Account>().unwrap());
                        } else {
                            black_box(black_box(&text).parse::<ExternalAccount>().unwrap());
                        }
                    });
                    measure(&format!("{label}-{run}-{implementation}-format"), || {
                        if candidate {
                            black_box(black_box(&owned).to_string());
                        } else {
                            black_box(black_box(&external).to_string());
                        }
                    });
                }
            }
        }
    }
}
