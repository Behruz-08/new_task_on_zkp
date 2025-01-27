// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use math::FieldElement;
use utils::collections::Vec;

// AUXILIARY TRACE SEGMENT RANDOMNESS
// ================================================================================================

/// Random elements used in construction of auxiliary trace segments.
///
/// These elements are generated by the
/// [Air::get_aux_trace_segment_random_elements()](crate::Air::get_aux_trace_segment_random_elements)
/// function for each auxiliary trace segment. In the interactive version of the protocol, the
/// verifier draws these elements uniformly at random from the extension field of the protocol
/// after the prover commits to a previous trace segment.
#[derive(Debug, Clone)]
pub struct AuxTraceRandElements<E: FieldElement>(Vec<Vec<E>>);

impl<E: FieldElement> AuxTraceRandElements<E> {
    /// Instantiates and returns an empty set of random elements.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns a list of random elements for an auxiliary segment with the specified index.
    pub fn get_segment_elements(&self, aux_segment_idx: usize) -> &[E] {
        &self.0[aux_segment_idx]
    }

    /// Adds random elements for a new auxiliary segment to this set of random elements.
    pub fn add_segment_elements(&mut self, rand_elements: Vec<E>) {
        self.0.push(rand_elements);
    }
}

impl<E: FieldElement> Default for AuxTraceRandElements<E> {
    fn default() -> Self {
        Self::new()
    }
}

// CONSTRAINT COMPOSITION COEFFICIENTS
// ================================================================================================
/// Coefficients used in construction of constraint composition polynomial.
///
/// These coefficients are created by the
/// [Air::get_constraint_composition_coefficients()](crate::Air::get_constraint_composition_coefficients)
/// function. In the interactive version of the protocol, the verifier draws these coefficients
/// uniformly at random from the extension field of the protocol.
///
/// There is one coefficient for each constraint so that we can compute a random linear
/// combination of constraints as:
/// $$
/// \sum_{i = 0}^k{\alpha_i \cdot C_i(x)}
/// $$
/// where:
/// * $\alpha_i$ is the coefficient for the $i$th constraint.
/// * $C_i(x)$ is an evaluation of the $i$th constraint at $x$.
///
/// The coefficients are separated into two lists: one for transition constraints and another one
/// for boundary constraints. This separation is done for convenience only.
#[derive(Debug, Clone)]
pub struct ConstraintCompositionCoefficients<E: FieldElement> {
    pub transition: Vec<E>,
    pub boundary: Vec<E>,
}

// DEEP COMPOSITION COEFFICIENTS
// ================================================================================================
/// Coefficients used in construction of DEEP composition polynomial.
///
/// These coefficients are created by the
/// [Air::get_deep_composition_coefficients()](crate::Air::get_deep_composition_coefficients)
/// function. In the interactive version of the protocol, the verifier draws these coefficients
/// uniformly at random from the extension field of the protocol.
///
/// The coefficients are used in computing the DEEP composition polynomial as:
/// $$
/// Y(x) = \sum_{i=0}^k{(
///     \alpha_i \cdot (\frac{T_i(x) - T_i(z)}{x - z} +
///     \frac{T_i(x) - T_i(z \cdot g)}{x - z \cdot g})
/// )} + \sum_{j=0}^m{\beta_j \cdot \frac{H_j(x) - H_j(z)}{x - z}}
/// $$
/// where:
/// * $z$ is an out-of-domain point drawn randomly from the entire field. In the interactive
///   version of the protocol, $z$ is provided by the verifier.
/// * $g$ is the generator of the trace domain. This is the $n$th root of unity where
///   $n$ is the length of the execution trace.
/// * $T_i(x)$ is an evaluation of the $i$th trace polynomial at $x$, and $k$ is the total
///   number of trace polynomials (which is equal to the width of the execution trace).
/// * $H_i(x)$ is an evaluation of the $j$th constraint composition column polynomial at $x$,
///   and $m$ is the total number of column polynomials.
/// * $\alpha_i$ is a composition coefficient for the $i$th trace polynomial.
/// * $\beta_j$ is a composition coefficient for the $j$th constraint column polynomial.
///
/// The soundness of the resulting protocol with batching as above is given in Theorem 8 in
/// https://eprint.iacr.org/2022/1216 and it relies on two points:
///
/// 1. The evaluation proofs for each trace polynomial at $z$ and $g \cdot z$ can be batched using
/// the non-normalized Lagrange kernel over the set $\{z, g \cdot z\}$. This, however, requires
/// that the FRI protocol is run with rate $\rho^{+} := \frac{\kappa + 2}{\nu}$ where $\kappa$ and
/// $\nu$ are the length of the execution trace and the LDE domain size, respectively.
/// 2. The resulting $Y(x)$ do not need to be degree adjusted but the soundness error of the
/// protocol needs to be updated. For most combinations of batching parameters, this leads to a
/// negligible increase in soundness error. The formula for the updated error can be found in
/// Theorem 8 of https://eprint.iacr.org/2022/1216.
#[derive(Debug, Clone)]
pub struct DeepCompositionCoefficients<E: FieldElement> {
    /// Trace polynomial composition coefficients $\alpha_i$.
    pub traces: Vec<Vec<E>>,
    /// Constraint column polynomial composition coefficients $\beta_j$.
    pub constraints: Vec<E>,
}
