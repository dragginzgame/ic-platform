use super::*;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    error::Error,
    panic::{AssertUnwindSafe, catch_unwind},
};

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_owned()
    } else {
        panic!("expected a textual panic payload")
    }
}

#[test]
fn spare_capacity_copy_publishes_exact_initialized_bytes_including_empty() {
    for size in [0, 1, 29, 64, 1025] {
        let calls = Cell::new(0);
        let bytes = system_bytes(size, |output, offset| {
            calls.set(calls.get() + 1);
            assert_eq!(offset, 0);
            assert_eq!(output.len(), size);
            for (index, byte) in output.iter_mut().enumerate() {
                byte.write(u8::try_from(index % 251).unwrap());
            }
        });
        let expected: Vec<u8> = (0..size)
            .map(|index| u8::try_from(index % 251).unwrap())
            .collect();
        assert_eq!(bytes, expected);
        assert_eq!(calls.get(), 1);
    }
}

#[test]
fn interrupted_copy_never_returns_partially_initialized_storage() {
    let returned = Cell::new(false);
    let result = catch_unwind(AssertUnwindSafe(|| {
        system_bytes(8, |output, offset| {
            assert_eq!(offset, 0);
            output[0].write(0xab);
            panic!("copy interrupted");
        });
        returned.set(true);
    }));
    assert_eq!(panic_message(result.unwrap_err()), "copy interrupted");
    assert!(!returned.get());
}

#[test]
fn text_validation_is_strict_and_omits_rejected_bytes_from_panics() {
    for text in ["", "method", "Réponse 🦀", "\0"] {
        assert_eq!(system_text(text.as_bytes().to_vec(), "invalid"), text);
    }
    for reason in [
        "reject message is not valid UTF-8",
        "msg_method_name is not valid UTF-8",
    ] {
        for bytes in [
            vec![0xff],
            vec![0xc3],
            vec![0xe2, 0x28, 0xa1],
            b"private\xff".to_vec(),
        ] {
            let result = catch_unwind(|| system_text(bytes, reason));
            assert_eq!(panic_message(result.unwrap_err()), reason);
        }
    }
}

#[test]
fn principal_copy_admits_every_length_and_refuses_oversize_before_copy() {
    for size in 0..=29 {
        let calls = Cell::new(0);
        let principal = system_principal(size, |output, offset| {
            calls.set(calls.get() + 1);
            assert_eq!(offset, 0);
            assert_eq!(output.len(), size);
            for (index, byte) in output.iter_mut().enumerate() {
                *byte = u8::try_from(index + 1).unwrap();
            }
        });
        let expected: Vec<u8> = (1..=size)
            .map(|value| u8::try_from(value).unwrap())
            .collect();
        assert_eq!(principal.as_slice(), expected);
        assert_eq!(calls.get(), 1);
    }
    for size in [30, 256, usize::MAX] {
        let copied = Cell::new(false);
        let result = catch_unwind(AssertUnwindSafe(|| {
            system_principal(size, |_, _| copied.set(true))
        }));
        assert!(result.is_err());
        assert!(!copied.get());
    }
}

#[test]
fn certificate_absence_skips_size_and_copy_but_present_empty_calls_both() {
    assert_eq!(
        certificate_bytes(
            0,
            || panic!("absent size query"),
            |_, _| panic!("absent copy")
        ),
        None
    );
    for present in [1, 2, u32::MAX] {
        let calls = RefCell::new(Vec::new());
        let certificate = certificate_bytes(
            present,
            || {
                calls.borrow_mut().push("size");
                0
            },
            |output, offset| {
                assert!(output.is_empty());
                assert_eq!(offset, 0);
                calls.borrow_mut().push("copy");
            },
        );
        assert_eq!(certificate, Some(Vec::new()));
        assert_eq!(*calls.borrow(), ["size", "copy"]);
    }
    let certificate = certificate_bytes(
        1,
        || 3,
        |output, offset| {
            assert_eq!(offset, 0);
            for (slot, value) in output.iter_mut().zip([0, 0xff, 7]) {
                slot.write(value);
            }
        },
    );
    assert_eq!(certificate, Some(vec![0, 0xff, 7]));
}

#[test]
fn system_boolean_refuses_non_boolean_supplier_values_with_exact_reason() {
    for operation in ["is_controller", "in_replicated_execution"] {
        assert!(!system_bool(0, operation));
        assert!(system_bool(1, operation));
        for value in [2, u32::MAX] {
            let result = catch_unwind(|| system_bool(value, operation));
            assert_eq!(
                panic_message(result.unwrap_err()),
                format!("unexpected return value from {operation}: {value}")
            );
        }
    }
}

#[test]
fn performance_counter_projections_preserve_payload_and_variant_distinctions() {
    for value in [0, 1, 2, 17, u32::MAX] {
        let counter = PerformanceCounterType::from(value);
        assert_eq!(u32::from(counter), value);
        assert_eq!(counter, value);
        match value {
            0 => assert_eq!(counter, PerformanceCounterType::InstructionCounter),
            1 => assert_eq!(
                counter,
                PerformanceCounterType::CallContextInstructionCounter
            ),
            _ => assert_eq!(counter, PerformanceCounterType::Unrecognized(value)),
        }
    }
    for (value, named) in [
        (0, PerformanceCounterType::InstructionCounter),
        (1, PerformanceCounterType::CallContextInstructionCounter),
    ] {
        let explicit = PerformanceCounterType::Unrecognized(value);
        assert_eq!(u32::from(explicit), value);
        assert_eq!(explicit, value);
        assert_ne!(explicit, named);
    }
}

#[test]
fn signing_cost_retains_full_amounts_and_error_codes() {
    for amount in [
        0,
        1,
        u128::from(u64::MAX),
        u128::from(u64::MAX) + 1,
        u128::MAX,
    ] {
        assert_eq!(sign_cost_result(amount, 0).unwrap(), amount);
    }
    assert!(matches!(
        sign_cost_result(u128::MAX, 1),
        Err(SignCostError::InvalidCurveOrAlgorithm)
    ));
    assert!(matches!(
        sign_cost_result(u128::MAX, 2),
        Err(SignCostError::InvalidKeyName)
    ));
    for code in [3, 255, u32::MAX] {
        match sign_cost_result(u128::MAX, code) {
            Err(SignCostError::UnrecognizedError(retained)) => assert_eq!(retained, code),
            other => panic!("unexpected cost mapping: {other:?}"),
        }
    }
    for (error, display) in [
        (
            SignCostError::InvalidCurveOrAlgorithm,
            "invalid curve or algorithm",
        ),
        (SignCostError::InvalidKeyName, "invalid key name"),
        (
            SignCostError::UnrecognizedError(u32::MAX),
            "unrecognized error: 4294967295",
        ),
    ] {
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
        assert_eq!(format!("{:?}", error.clone()), format!("{error:?}"));
    }
}
