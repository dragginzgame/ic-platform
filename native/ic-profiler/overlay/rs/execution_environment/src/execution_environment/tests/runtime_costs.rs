use super::{
    get_reject_message, make_bip340_key, make_ecdsa_key, make_vetkd_key,
    sign_with_threshold_key_payload,
};
use ic_interfaces::execution_environment::{
    RuntimeCostKind, TracedRuntimeCost, TracedWasmExecution, WasmMemoryAccessTracer,
};
use ic_management_canister_types_private::{
    BoundedHttpHeaders, CanisterHttpRequestArgs, HttpMethod, MasterPublicKeyId, Method,
    PRICING_VERSION_LEGACY, PRICING_VERSION_PAY_AS_YOU_GO, Payload as Ic00Payload,
};
use ic_test_utilities_execution_environment::{
    ExecutionTest, ExecutionTestBuilder, get_output_messages,
};
use ic_test_utilities_types::ids::{canister_test_id, subnet_test_id};
use ic_types::canister_http::PricingVersion;
use ic_types::{CanisterId, SubnetId};
use ic_types_cycles::{CanisterCyclesCostSchedule, Cycles, CyclesUseCase};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct ServiceCostRecorder {
    events: Mutex<Vec<(CanisterId, TracedRuntimeCost)>>,
}

impl WasmMemoryAccessTracer for ServiceCostRecorder {
    fn is_tracing_enabled(&self, _canister_id: CanisterId) -> bool {
        true
    }

    fn record(&self, _canister_id: CanisterId, _execution: TracedWasmExecution) {}

    fn record_runtime_cost(&self, canister_id: CanisterId, cost: TracedRuntimeCost) {
        self.events.lock().unwrap().push((canister_id, cost));
    }
}

// Run through the public management dispatcher, including decoding, validation,
// context insertion and actual attached-payment deductions. Only the observer
// differs between the two harnesses; no Wasm or public cost-quote API is needed.
fn service_builder(
    schedule: CanisterCyclesCostSchedule,
    source_subnet: SubnetId,
) -> ExecutionTestBuilder {
    ExecutionTestBuilder::new()
        .with_subnet_size(13)
        .with_cost_schedule(schedule)
        .with_caller(source_subnet, canister_test_id(16))
        .with_flexible_http_requests_enabled()
}

fn enable_http(test: &mut ExecutionTest) {
    Arc::make_mut(&mut test.state_mut().metadata.own_subnet_info)
        .subnet_features
        .http_requests = true;
}

fn http_args(pricing: &PricingVersion, non_replicated: bool) -> Vec<u8> {
    CanisterHttpRequestArgs {
        url: "https://example.com/resource".into(),
        max_response_bytes: Some(1_024),
        headers: BoundedHttpHeaders::new(vec![]),
        body: Some(vec![7; 13]),
        method: HttpMethod::POST,
        transform: None,
        is_replicated: Some(!non_replicated),
        pricing_version: Some(match pricing {
            PricingVersion::Legacy => PRICING_VERSION_LEGACY,
            PricingVersion::PayAsYouGo => PRICING_VERSION_PAY_AS_YOU_GO,
        }),
    }
    .encode()
}

#[test]
fn initial_http_fees_trace_committed_payment_without_changing_state() {
    for schedule in [
        CanisterCyclesCostSchedule::Normal,
        CanisterCyclesCostSchedule::Free,
    ] {
        for pricing in [PricingVersion::Legacy, PricingVersion::PayAsYouGo] {
            for non_replicated in [false, true] {
                let recorder = Arc::new(ServiceCostRecorder::default());
                // HTTP outcalls are local management calls; their permission
                // check rejects callers routed to another subnet.
                let mut expected = service_builder(schedule, subnet_test_id(1)).build();
                let mut observed = service_builder(schedule, subnet_test_id(1))
                    .with_runtime_cost_tracer(recorder.clone())
                    .build();
                enable_http(&mut expected);
                enable_http(&mut observed);
                let payload = http_args(&pricing, non_replicated);
                let payment = Cycles::new(1_000_000_000_000);
                expected.inject_call_to_ic00(Method::HttpRequest, payload.clone(), payment);
                observed.inject_call_to_ic00(Method::HttpRequest, payload.clone(), payment);
                assert!(expected.execute_subnet_message());
                assert!(observed.execute_subnet_message());
                assert_eq!(observed.state(), expected.state());
                let stored = observed
                    .state()
                    .metadata
                    .subnet_call_context_manager
                    .canister_http_request_contexts
                    .values()
                    .next()
                    .expect("HTTP request must be accepted");
                let actual_pricing = &stored.pricing_version;
                assert_eq!(actual_pricing, &pricing);
                let actual_allowance = if *actual_pricing == PricingVersion::PayAsYouGo {
                    stored.refund_status.refundable_cycles
                } else {
                    Cycles::zero()
                };
                let actual_fee = payment - stored.request.payment - actual_allowance;
                let nominal_fee = observed
                    .state()
                    .metadata
                    .subnet_metrics
                    .get_consumed_cycles_http_outcalls()
                    .get();
                let events = recorder.events.lock().unwrap();
                assert_eq!(events.len(), 1 + usize::from(!actual_allowance.is_zero()));
                let (caller, fee) = &events[0];
                assert_eq!(*caller, canister_test_id(16));
                assert_eq!(fee.rule_id, "ic.cycles.http.v1");
                assert_eq!(fee.kind, RuntimeCostKind::Debit);
                assert_eq!(fee.amount, actual_fee.get());
                assert_eq!(fee.nominal_cycles, Some(nominal_fee));
                assert!(nominal_fee > 0);
                if schedule == CanisterCyclesCostSchedule::Free {
                    assert_eq!(fee.amount, 0);
                } else {
                    assert!(fee.amount > 0);
                }
                assert_eq!(
                    fee.callback_id,
                    Some(stored.request.sender_reply_callback.get())
                );
                assert_eq!(fee.payload_bytes, Some(stored.variable_parts_size().get()));
                assert_eq!(
                    fee.details["method_payload_bytes"],
                    payload.len().to_string()
                );
                assert_eq!(fee.details["pricing_version"], actual_pricing.as_str());
                assert_eq!(fee.details["accounting_domain"], "http_service_allowance");
                if !actual_allowance.is_zero() {
                    let (_, reservation) = &events[1];
                    assert_eq!(reservation.rule_id, "ic.cycles.http_allowance.v1");
                    assert_eq!(reservation.kind, RuntimeCostKind::Reservation);
                    assert_eq!(reservation.amount, actual_allowance.get());
                    assert_eq!(reservation.nominal_cycles, None);
                    assert_eq!(
                        reservation.details["accounting_domain"],
                        "http_service_allowance"
                    );
                }
            }
        }
    }
}

#[test]
fn threshold_fees_trace_committed_payment_and_exemptions_without_changing_state() {
    for key in [
        make_ecdsa_key("cost"),
        make_bip340_key("cost"),
        make_vetkd_key("cost"),
    ] {
        let (method, use_case, rule_id) = match &key {
            MasterPublicKeyId::Ecdsa(_) => (
                Method::SignWithECDSA,
                CyclesUseCase::ECDSAOutcalls,
                "ic.cycles.ecdsa.v1",
            ),
            MasterPublicKeyId::Schnorr(_) => (
                Method::SignWithSchnorr,
                CyclesUseCase::SchnorrOutcalls,
                "ic.cycles.schnorr.v1",
            ),
            MasterPublicKeyId::VetKd(_) => (
                Method::VetKdDeriveKey,
                CyclesUseCase::VetKd,
                "ic.cycles.vetkd.v1",
            ),
        };
        for schedule in [
            CanisterCyclesCostSchedule::Normal,
            CanisterCyclesCostSchedule::Free,
        ] {
            for nns_exempt in [false, true] {
                let source_subnet = subnet_test_id(if nns_exempt { 2 } else { 3 });
                let recorder = Arc::new(ServiceCostRecorder::default());
                let mut expected = service_builder(schedule, source_subnet)
                    .with_chain_key(key.clone())
                    .build();
                let mut observed = service_builder(schedule, source_subnet)
                    .with_chain_key(key.clone())
                    .with_runtime_cost_tracer(recorder.clone())
                    .build();
                let payload = sign_with_threshold_key_payload(method, key.clone());
                let payment = Cycles::new(100_000_000_000);
                expected.inject_call_to_ic00(method, payload.clone(), payment);
                observed.inject_call_to_ic00(method, payload, payment);
                assert!(expected.execute_subnet_message());
                assert!(observed.execute_subnet_message());
                assert_eq!(observed.state(), expected.state());
                let stored = observed
                    .state()
                    .metadata
                    .subnet_call_context_manager
                    .sign_with_threshold_contexts
                    .values()
                    .next()
                    .expect("Threshold request must be accepted");
                let nominal_fee = observed
                    .state()
                    .metadata
                    .subnet_metrics
                    .get_consumed_cycles_by_use_case()
                    .get(&use_case)
                    .map_or(0, |fee| fee.get());
                let events = recorder.events.lock().unwrap();
                assert_eq!(events.len(), 1);
                let (caller, event) = &events[0];
                assert_eq!(*caller, canister_test_id(16));
                assert_eq!(event.rule_id, rule_id);
                assert_eq!(event.kind, RuntimeCostKind::Debit);
                assert_eq!(event.amount, (payment - stored.request.payment).get());
                assert_eq!(
                    event.amount,
                    if nns_exempt || schedule == CanisterCyclesCostSchedule::Free {
                        0
                    } else {
                        10_000_000_000
                    }
                );
                assert_eq!(event.nominal_cycles, Some(nominal_fee));
                assert_eq!(nominal_fee, if nns_exempt { 0 } else { 10_000_000_000 });
                assert_eq!(
                    event.callback_id,
                    Some(stored.request.sender_reply_callback.get())
                );
                assert_eq!(event.details["nns_exempt"], nns_exempt.to_string());
                assert_eq!(event.details["key_id"], key.to_string());
                assert_eq!(event.details["source_subnet"], source_subnet.to_string());
            }
        }
    }
}

#[test]
fn rejected_native_service_requests_emit_no_fee_events() {
    let key = make_ecdsa_key("cost");
    for scenario in [
        "http_unpaid",
        "http_full",
        "signature_unpaid",
        "signature_disabled",
    ] {
        let recorder = Arc::new(ServiceCostRecorder::default());
        let builder = || {
            let source_subnet = subnet_test_id(if scenario.starts_with("http_") { 1 } else { 3 });
            let builder = service_builder(CanisterCyclesCostSchedule::Normal, source_subnet);
            match scenario {
                "http_full" => builder.with_max_canister_http_requests_in_flight(0),
                "signature_disabled" => builder.with_disabled_chain_key(key.clone()),
                _ => builder.with_chain_key(key.clone()),
            }
        };
        let mut expected = builder().build();
        let mut observed = builder().with_runtime_cost_tracer(recorder.clone()).build();
        enable_http(&mut expected);
        enable_http(&mut observed);
        let (method, payload) = if scenario.starts_with("http_") {
            (
                Method::HttpRequest,
                http_args(&PricingVersion::Legacy, false),
            )
        } else {
            (
                Method::SignWithECDSA,
                sign_with_threshold_key_payload(Method::SignWithECDSA, key.clone()),
            )
        };
        let payment = if scenario.ends_with("unpaid") {
            Cycles::zero()
        } else {
            Cycles::new(1_000_000_000_000)
        };
        expected.inject_call_to_ic00(method, payload.clone(), payment);
        observed.inject_call_to_ic00(method, payload, payment);
        assert!(expected.execute_subnet_message());
        assert!(observed.execute_subnet_message());
        assert_eq!(observed.state(), expected.state());
        assert!(
            observed
                .state()
                .metadata
                .subnet_call_context_manager
                .canister_http_request_contexts
                .is_empty()
        );
        assert!(
            observed
                .state()
                .metadata
                .subnet_call_context_manager
                .sign_with_threshold_contexts
                .is_empty()
        );
        assert!(
            recorder.events.lock().unwrap().is_empty(),
            "Rejected {scenario} must not emit fee events"
        );
        let replies = get_output_messages(observed.state_mut());
        assert_eq!(replies.len(), 1);
        let reason = get_reject_message(replies[0].1.clone());
        let expected_reason = match scenario {
            "http_full" => "max number (0) of http requests in-flight reached",
            "signature_disabled" => "unknown or disabled threshold key",
            _ => "request sent with 0 cycles",
        };
        assert!(
            reason.contains(expected_reason),
            "Unexpected rejection for {scenario}: {reason}"
        );
    }
}
