use super::*;
use ic_config::embedders::DEFAULT_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES;
#[test]
fn instrumented_origin_map_cap_reports_missing_ranges_without_extending_a_neighbor() {
    let mut module = wasm_encoder::Module::new();
    let mut types = wasm_encoder::TypeSection::new();
    types.ty().function([], []);
    module.section(&types);
    let mut functions = wasm_encoder::FunctionSection::new();
    functions.function(0);
    module.section(&functions);
    let mut function = wasm_encoder::Function::new([]);
    let count = DEFAULT_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES + 2;
    for _ in 0..count {
        function.instruction(&wasm_encoder::Instruction::Nop);
    }
    function.instruction(&wasm_encoder::Instruction::End);
    let mut code = wasm_encoder::CodeSection::new();
    code.function(&function);
    module.section(&code);
    let wasm = module.finish();
    let tags = vec![
        (0..=count)
            .map(|id| OriginTag {
                origin_id: Some(id as u32),
                kind: "original",
            })
            .collect(),
    ];
    let mut metadata = InstructionProfilingMetadata::default();
    export_instrumented_origins(
        &wasm,
        &tags,
        &mut metadata,
        DEFAULT_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES,
    );
    assert!(metadata.origin_map_truncated);
    assert_eq!(
        metadata.instrumented_ranges.len(),
        DEFAULT_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES
    );
    let omitted = metadata.instrumented_ranges.last().unwrap().end_offset;
    assert!(omitted < (wasm.len() - 1) as u64);
    assert!(
        metadata
            .instrumented_ranges
            .iter()
            .all(|range| !(range.start_offset <= omitted && omitted < range.end_offset))
    );
}

#[test]
fn instrumented_origin_map_limit_counts_merged_ranges_not_operators() {
    let mut module = wasm_encoder::Module::new();
    let mut types = wasm_encoder::TypeSection::new();
    types.ty().function([], []);
    module.section(&types);
    let mut functions = wasm_encoder::FunctionSection::new();
    functions.function(0);
    module.section(&functions);
    let mut function = wasm_encoder::Function::new([]);
    function.instruction(&wasm_encoder::Instruction::Nop);
    function.instruction(&wasm_encoder::Instruction::Nop);
    function.instruction(&wasm_encoder::Instruction::End);
    let mut code = wasm_encoder::CodeSection::new();
    code.function(&function);
    module.section(&code);
    let wasm = module.finish();
    let tags = vec![vec![
        OriginTag {
            origin_id: None,
            kind: "runtime_helper",
        };
        3
    ]];
    let mut metadata = InstructionProfilingMetadata::default();
    export_instrumented_origins(&wasm, &tags, &mut metadata, 1);
    assert!(!metadata.origin_map_truncated);
    assert_eq!(metadata.instrumented_ranges.len(), 1);
    assert_eq!(
        metadata.instrumented_ranges[0].end_offset,
        wasm.len() as u64
    );

    let mut omitted = InstructionProfilingMetadata::default();
    export_instrumented_origins(&wasm, &tags, &mut omitted, 0);
    assert!(omitted.origin_map_truncated);
    assert!(omitted.instrumented_ranges.is_empty());
}
