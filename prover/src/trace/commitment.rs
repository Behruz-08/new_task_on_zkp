// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::TraceLde;
use crate::RowMatrix;
use air::proof::JointTraceQueries;
use crypto::{ElementHasher, MerkleTree};
use math::FieldElement;
use utils::collections::Vec;

// TRACE COMMITMENT
// ================================================================================================

/// Execution trace commitment.
///
/// The describes one or more trace segments, each consisting of the following components:
/// * Evaluations of a trace segment's polynomials over the LDE domain.
/// * Merkle tree where each leaf in the tree corresponds to a row in the trace LDE matrix.
pub struct TraceCommitment<E: FieldElement, H: ElementHasher<BaseField = E::BaseField>> {
    traces_lde: Vec<TraceLde<E>>,
    main_segment_tree: MerkleTree<H>,
    //We most probably need just one aux_tree
    aux_segment_trees: Vec<MerkleTree<H>>,
}

impl<E: FieldElement, H: ElementHasher<BaseField = E::BaseField>> TraceCommitment<E, H> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Creates a new trace commitment from the provided main trace low-degree extension and the
    /// corresponding Merkle tree commitment.
    pub fn new(
        main_traces_lde: Vec<RowMatrix<E::BaseField>>,
        main_trace_tree: MerkleTree<H>,
        blowups: Vec<usize>,
    ) -> Self {
        let mut num_rows_for_traces = main_traces_lde.iter().map(|main_trace_lde| main_trace_lde.num_rows());
        let main_tree_leaves_lenght = main_trace_tree.leaves().len();
        assert!(num_rows_for_traces.all(|num_rows| num_rows == main_tree_leaves_lenght), "number of rows in trace LDE must be the same as number of leaves in trace commitment");
        let traces_lde = main_traces_lde.iter().zip(blowups).map(|(&main_trace_lde, blowup)| TraceLde::new(main_trace_lde, blowup)).collect();
        Self {
            traces_lde,
            main_segment_tree: main_trace_tree,
            aux_segment_trees: Vec::new(),
        }
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Adds the provided auxiliary segment trace LDE and Merkle tree to this trace commitment.
    pub fn add_segment(&mut self, aux_segment_lde: RowMatrix<E>, aux_segment_tree: MerkleTree<H>) {
        assert_eq!(
            aux_segment_lde.num_rows(),
            aux_segment_tree.leaves().len(),
            "number of rows in trace LDE must be the same as number of leaves in trace commitment"
        );

        self.trace_lde.add_aux_segment(aux_segment_lde);
        self.aux_segment_trees.push(aux_segment_tree);
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the execution trace for this commitment.
    ///
    /// The trace contains both the main trace segment and the auxiliary trace segments (if any).
    pub fn trace_table(&self) -> &TraceLde<E> {
        &self.trace_lde
    }
    pub fn trace1_table(&self) -> &TraceLde<E> {
        &self.trace1_lde
    }

    // QUERY TRACE
    // --------------------------------------------------------------------------------------------
    /// Returns trace table rows at the specified positions along with Merkle authentication paths
    /// from the commitment root to these rows.
    pub fn query(&self, positions: &[usize]) -> Vec<JointTraceQueries> {
        // build queries for the main trace segment
        let mut result = vec![build_segment_queries(
            self.trace_lde.get_main_segment(),
            self.trace1_lde.get_main_segment(),
            &self.main_segment_tree,
            positions,
        )];

        // build queries for auxiliary trace segments
        for (i, segment_tree) in self.aux_segment_trees.iter().enumerate() {
            let segment_lde = self.trace_lde.get_aux_segment(i);
            let segment1_lde = self.trace1_lde.get_aux_segment(i);
            result.push(build_segment_queries(
                segment_lde,
                segment1_lde,
                segment_tree,
                positions,
            ));
        }

        result
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    /// Returns the root of the commitment Merkle tree.
    #[cfg(test)]
    pub fn main_trace_root(&self) -> H::Digest {
        *self.main_segment_tree.root()
    }

    /// Returns the entire trace for the column at the specified index.
    #[cfg(test)]
    pub fn get_main_trace_column(&self, col_idx: usize) -> Vec<E::BaseField> {
        let trace = self.trace_lde.get_main_segment();
        (0..trace.num_rows())
            .map(|row_idx| trace.get(col_idx, row_idx))
            .collect()
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn build_segment_queries<E, H>(
    segment_lde: &RowMatrix<E>,
    segment1_lde: &RowMatrix<E>,
    segment_tree: &MerkleTree<H>,
    positions: &[usize],
) -> JointTraceQueries
where
    E: FieldElement,
    H: ElementHasher<BaseField = E::BaseField>,
{
    // for each position, get the corresponding row from the trace segment LDE and put all these
    // rows into a single vector
    let trace_states = positions
        .iter()
        .map(|&pos| segment_lde.row(pos).to_vec())
        .collect::<Vec<_>>();
    let trace1_states = positions
        .iter()
        .map(|&pos| segment1_lde.row(pos).to_vec())
        .collect::<Vec<_>>();
    /*let comb_states = trace_states
    .iter()
    .zip(trace1_states.iter())
    .map(|(&ref row, &ref row1)| [row, row1].concat())
    .collect::<Vec<_>>();*/
    let mut comb_states = Vec::with_capacity(trace_states.len());
    for (i, row) in trace_states.iter().enumerate() {
        let trace_row = &row[..];
        let trace1_row = &trace1_states[i][..];
        comb_states.push([trace_row, trace1_row].concat());
    }
    // build Merkle authentication paths to the leaves specified by positions
    let trace_proof = segment_tree
        .prove_batch(positions)
        .expect("failed to generate a Merkle proof for trace queries");

    JointTraceQueries::new(trace_proof, comb_states, trace_states, trace1_states)
}
