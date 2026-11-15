#![forbid(unsafe_code)]

//! Public raw-argument admission fixture, executed by an independent Wasm host.

#[cfg(target_arch = "wasm32")]
mod runtime_arguments;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use candidate_runtime::system::{ArgumentTooLarge, read_args};

        assert_eq!(read_args(0), Ok(vec![]));
        let retained = read_args(3).unwrap();
        assert_eq!(retained, [0, 128, 255]);
        assert_eq!(
            read_args(2),
            Err(ArgumentTooLarge {
                actual: 3,
                maximum: 2
            })
        );
        assert_eq!(
            read_args(0),
            Err(ArgumentTooLarge {
                actual: 1,
                maximum: 0
            })
        );
        assert_eq!(
            read_args(4),
            Err(ArgumentTooLarge {
                actual: u32::MAX as usize,
                maximum: 4
            })
        );
        assert_eq!(read_args(10), Ok(vec![7, 8]));
        assert_eq!(retained, [0, 128, 255]);
        runtime_arguments::guarded();
    }
}
