#![cfg_attr(not(feature = "std"), no_std)]

use core::{fmt, future::Future, str::FromStr};

use starknet_types_core::felt::Felt;

// FIXME: most of the types here do not have any documentation
// In all of the crate that defines them (mainly stone-prover-sdk and cairo-vm, those types
// are not documented at all).
// The stone prover itself doesn't really have any documentation either, and it doesn't seem
// to use the public input or the private input in a meaningful way within the codebase that's
// opensourced.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layout {
    Plain,
    Small,
    Dex,
    Recursive,
    Starknet,
    RecursiveLargeOutput,
    AllCairo,
    AllSolidity,
    StarknetWithKeccak,
}

impl Layout {
    /// Returns the name of the layout as a string.
    ///
    /// The string uses snake case, as it is defined in the original stone prover implementation.
    pub const fn name(self) -> &'static str {
        match self {
            Layout::Plain => "plain",
            Layout::Small => "small",
            Layout::Dex => "dex",
            Layout::Recursive => "recursive",
            Layout::Starknet => "starknet",
            Layout::RecursiveLargeOutput => "recursive_large_output",
            Layout::AllCairo => "all_cairo",
            Layout::AllSolidity => "all_solidity",
            Layout::StarknetWithKeccak => "starknet_with_keccak",
        }
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

impl FromStr for Layout {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Layout::Plain),
            "small" => Ok(Layout::Small),
            "dex" => Ok(Layout::Dex),
            "recursive" => Ok(Layout::Recursive),
            "starknet" => Ok(Layout::Starknet),
            "recursive_large_output" => Ok(Layout::RecursiveLargeOutput),
            "all_cairo" => Ok(Layout::AllCairo),
            "all_solidity" => Ok(Layout::AllSolidity),
            "starknet_with_keccak" => Ok(Layout::StarknetWithKeccak),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemorySegment<'a> {
    pub name: &'a str,
    pub start: Felt,
    pub end: Felt,
}

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub address: usize,
    pub value: Felt,
    pub page: usize,
}

/// The request type passed to the [`Prove::prove`] method.
///
/// This type contains all necessary information that the prover needs to generate a proof.
#[derive(Debug, Clone)]
pub struct ProofRequest<'a> {
    //
    // PUBLIC INPUT
    //
    pub layout: Layout,
    pub rc_min: isize,
    pub rc_max: isize,
    pub n_steps: usize,
    pub memory_segments: &'a [MemorySegment<'a>],
    pub public_memory: &'a [MemoryEntry],
    // FIXME: Find out how `dynamic_params` is supposed to be used.
    // There's a world where it's actually part of the `layout` field if I understood correctly.
    pub dynamic_params: (),
    //
    // PRIVATE INPUT
    //
    pub trace: &'a [u8],
    pub memory: &'a [u8],
    // FIXME: Find out the exact layout of each of those fields.
    pub pedersen: &'a [Felt],
    pub range_check: &'a [Felt],
    pub ecdsa: &'a [Felt],
    pub bitwise: &'a [Felt],
    pub ec_ops: &'a [Felt],
    pub keccak: &'a [Felt],
    pub poseidon: &'a [Felt],
}

/// The response produced by a prover. This is the return type produced by the [`Prove::prove`]
/// method.
#[derive(Debug, Clone)]
pub struct Proof {}

/// This trait encapsulates the behavior of a Starknet proving mechanism.
///
/// # Implementing
///
/// You will note that the [`prove`] method returns a [`Future`] that is not tied to the
/// lifetime of the input [`ProofRequest`] or `self`. This means that it is not possible for the
/// returned value to hold any references to any of those values. Instead, it must clone any data
/// it needs and move it into the returned [`Future`].
///
/// [`prove`]: Prove::prove
pub trait Prove {
    /// An error that might occur during the proving process.
    type Err;

    /// Generates a proof for the given [`ProofRequest`].
    fn prove(
        &mut self,
        request: &ProofRequest,
    ) -> impl Send + Future<Output = Result<Proof, Self::Err>>;
}
