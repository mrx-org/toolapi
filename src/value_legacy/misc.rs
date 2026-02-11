use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// MR signal. Outer `Vec`: list of channels/coils. Inner `Vec`: Complex ADC samples.
///
/// All channels are guaranteed to have the same number of samples.
/// Single-coil measurements have the shape `vec![vec![s1, s2, ...]]`
/// TODO: make member private and guarantee same sample count per coil on construction?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal(pub Vec<Vec<Complex64>>);

/// k-space encoding of a sequence.
///
/// Contains the `[kx, ky, kz, tau]` dephasing for every sample, which can be used
/// for e.g.: NUFFT reconstruction. `tau` specifies the amount of B0 / T2' dephasing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encoding(pub Vec<[f64; 4]>);
