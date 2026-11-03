// Compile the maintained example in the parent; the independent builder copies its exact sources.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/standalone/workflow/mod.rs"
));
