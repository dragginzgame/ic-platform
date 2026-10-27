/// Correlates observation-only routing events using protocol identities.
/// This exports sizes and identities, never message arguments or replies.
pub(crate) fn system_profile_call(
    operation: &str,
    message: &ic_types::messages::RequestOrResponse,
    source_subnet: ic_types::SubnetId,
    destination_subnet: ic_types::SubnetId,
    stream_index: Option<String>,
    status: &str,
) {
    if !ic_system_profiler::enabled() {
        return;
    }
    use ic_types::messages::RequestOrResponse;
    let mut fields = vec![
        ("sender", message.sender().to_string()),
        ("receiver", message.receiver().to_string()),
        ("source_subnet", source_subnet.to_string()),
        ("destination_subnet", destination_subnet.to_string()),
        ("status", status.to_string()),
        (
            "transport_kind",
            if source_subnet == destination_subnet {
                "same_subnet"
            } else {
                "xnet"
            }
            .to_string(),
        ),
    ];
    match message {
        RequestOrResponse::Request(request) => {
            fields.extend([
                ("message_kind", "request".to_string()),
                ("originator", request.sender.to_string()),
                (
                    "call_id",
                    format!("{}:{}", request.sender, request.sender_reply_callback),
                ),
                ("callback_id", request.sender_reply_callback.to_string()),
                ("method", request.method_name.clone()),
                (
                    "payload_bytes",
                    request.payload_size_bytes().get().to_string(),
                ),
                ("attached_cycles", request.payment.get().to_string()),
            ]);
        }
        RequestOrResponse::Response(response) => {
            fields.extend([
                ("message_kind", "response".to_string()),
                ("originator", response.originator.to_string()),
                (
                    "call_id",
                    format!(
                        "{}:{}",
                        response.originator, response.originator_reply_callback
                    ),
                ),
                (
                    "callback_id",
                    response.originator_reply_callback.to_string(),
                ),
                (
                    "payload_bytes",
                    response.payload_size_bytes().get().to_string(),
                ),
                ("refunded_cycles", response.refund.get().to_string()),
            ]);
        }
    }
    if let Some(index) = stream_index {
        fields.push(("stream_index", index));
    }
    ic_system_profiler::event_fields("ic.messaging.call", operation, &fields);
}
