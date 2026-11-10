//! Fixture-owned Wire identities and budgets shared by both RPC ends.

pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 64 * 1024,
    allocation: 1024 * 1024,
    nodes: 16 * 1024,
    depth: 32,
};
// One fixture declaration owns the fixed numeric IDs and wire method names.
macro_rules! methods {
    ($($variant:ident = $id:expr => $name:literal),* $(,)?) => {
        #[derive(Clone, Copy)]
        #[repr(u64)]
        pub enum Method { $($variant = $id,)* }
        pub fn method_id(name: &str) -> u64 {
            match name {
                $($name => Method::$variant as u64,)*
                _ => panic!("unknown fixture endpoint: {name}"),
            }
        }
    };
}
methods! {
    Initialize = 4096 => "initialize",
    AfterUpgrade = 4097 => "after_upgrade",
    Value = 4098 => "value",
    Scalar = 4099 => "scalar",
    Tuple = 4100 => "tuple",
    Unit = 4101 => "unit",
    Awaited = 4102 => "awaited",
    Composite = 4103 => "composite",
    Guarded = 4104 => "guarded",
    Manual = 4105 => "manual",
    CallbackTrap = 4106 => "callback_trap",
    Cleaned = 4107 => "cleaned",
    ProtectedPending = 4108 => "protected_pending",
    Migratory = 4109 => "migratory",
    Controller = 4110 => "controller",
    Missing = 4111 => "missing",
    CycleReceiver = 4112 => "cycle_receiver",
    CycleTotal = 4113 => "cycle_total",
    CycleRoundtrip = 4114 => "cycle_roundtrip",
    Checkpoint = 4115 => "checkpoint",
    StableEdit = 4116 => "stable_edit",
    StableValue = 4117 => "stable_value",
    CheckpointBytes = 4118 => "checkpoint_bytes",
    PublishRoot = 4119 => "publish_root",
    Certificate = 4120 => "certificate",
    SystemHelpers = 4121 => "system_helpers",
    ExternalManagement = 4122 => "external_management",
    SubmissionPressure = 0x2000 => "submission_pressure",
    InsufficientCycles = 0x2001 => "insufficient_cycles",
    WeakPending = 0x2002 => "weak_pending",
    BufferCopy = 0x2003 => "buffer_copy",
    SelfWakeQuery = 0x2004 => "self_wake_query",
    ManagementProof = 0x2005 => "management_proof",
    CancelTask = 0x2006 => "cancel_task",
    StableRefusedGrowth = 0x2007 => "stable_refused_growth",
    StableBoundsTrap = 0x2008 => "stable_bounds_trap",
    MultiplexedTrap = 0x2009 => "multiplexed_trap",
    MultiplexedCleaned = 0x200a => "multiplexed_cleaned",
    Absent = 0xffff => "no_such_method",
}
