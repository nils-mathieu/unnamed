# Information

The main package of hosted in this repository is a server responsible for proving incoming
starknet blocks and making them available to any data availability layer.

Currently, this repository hosts the following sub-crates:

1. The `trace` crate provides a common interface for generating execution traces and state
   updates for a given block.

2. The `starknet-prove-core` crate provides a common interface for generating proofs given a
   execution trace and a state update.

3. The `starknet-dal-core` crate provides a common interface for interacting with a data availability
   layer and making proofs available to it.

The point of those three crates is to abstract away the logic of generating and proving
execution traces, makingi t possible to swap out any of those components without having to
change the rest of the system.

Those libraries might move to their own repositories in the future depending on their size,
complexity, and usage outside of this project.

## State of the project

- Currently, the `trace` crate is not available. It always returns the same mock trace for testing
  purposes.

- `starknet-prove-stone` is using the stone prover in the background, as a child process.

That being said, adding backends to those crates is not hard, and require no changes in consumer
crates (which is the point of using those crates).
