use std::marker::PhantomData;

use bellperson::{ConstraintSystem, SynthesisError};
use fil_sapling_crypto::circuit::num;
use fil_sapling_crypto::jubjub::JubjubEngine;
use paired::bls12_381::{Bls12, Fr};

use crate::circuit::zigzag::{column_proof::ColumnProof, encoding_proof::EncodingProof};
use crate::circuit::{por::PoRCircuit, variables::Root};
use crate::drgraph::Graph;
use crate::hasher::Hasher;
use crate::merkle::MerkleProof;
use crate::zigzag::{
    Proof as VanillaProof, PublicParams, ReplicaColumnProof as VanillaReplicaColumnProof,
};

#[derive(Debug, Clone)]
pub struct Proof<H: Hasher> {
    pub comm_d_proof: InclusionPath<H>,
    pub comm_r_last_proofs: (InclusionPath<H>, Vec<InclusionPath<H>>),
    pub replica_column_proof: ReplicaColumnProof<H>,
    pub encoding_proof_1: EncodingProof,
    pub encoding_proofs: Vec<EncodingProof>,
}

impl<H: Hasher> Proof<H> {
    /// Create an empty proof, used in `blank_circuit`s.
    pub fn empty(params: &PublicParams<H>) -> Self {
        let degree = params.graph.degree();
        let challenges_count = params.layer_challenges.challenges_count();
        let layers = params.layer_challenges.layers();

        Proof {
            comm_d_proof: InclusionPath::empty(degree),
            comm_r_last_proofs: (
                InclusionPath::empty(degree),
                vec![InclusionPath::empty(degree); challenges_count],
            ),
            replica_column_proof: ReplicaColumnProof::empty(params),
            encoding_proof_1: EncodingProof::empty(params),
            encoding_proofs: vec![EncodingProof::empty(params); layers - 2],
        }
    }

    /// Circuit synthesis.
    pub fn synthesize<CS: ConstraintSystem<Bls12>>(
        self,
        mut cs: CS,
        params: &<Bls12 as JubjubEngine>::Params,
        comm_d: &num::AllocatedNum<Bls12>,
    ) -> Result<(), SynthesisError> {
        let Proof {
            comm_d_proof,
            comm_r_last_proofs,
            replica_column_proof,
            encoding_proof_1,
            encoding_proofs,
        } = self;

        // verify initial data layer
        comm_d_proof.synthesize(cs.namespace(|| "comm_d_inclusion"), params, comm_d)?;

        // verify replica column openings

        // verify final replica layer

        // verify encodings

        Ok(())
    }
}

impl<H: Hasher> From<VanillaProof<H>> for Proof<H> {
    fn from(vanilla_proof: VanillaProof<H>) -> Self {
        let VanillaProof {
            comm_d_proofs,
            comm_r_last_proofs,
            replica_column_proofs,
            encoding_proof_1,
            encoding_proofs,
        } = vanilla_proof;

        Proof {
            comm_d_proof: comm_d_proofs.into(),
            comm_r_last_proofs: (
                comm_r_last_proofs.0.into(),
                comm_r_last_proofs.1.into_iter().map(Into::into).collect(),
            ),
            replica_column_proof: replica_column_proofs.into(),
            encoding_proof_1: encoding_proof_1.into(),
            encoding_proofs: encoding_proofs.into_iter().map(|p| p.into()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InclusionPath<H: Hasher> {
    value: Option<Fr>,
    auth_path: Vec<Option<(Fr, bool)>>,
    _h: PhantomData<H>,
}

impl<H: Hasher> InclusionPath<H> {
    /// Create an empty proof, used in `blank_circuit`s.
    pub fn empty(degree: usize) -> Self {
        InclusionPath {
            value: None,
            auth_path: vec![None; degree],
            _h: PhantomData,
        }
    }

    pub fn synthesize<CS: ConstraintSystem<Bls12>>(
        self,
        cs: CS,
        params: &<Bls12 as JubjubEngine>::Params,
        root: &num::AllocatedNum<Bls12>,
    ) -> Result<(), SynthesisError> {
        let InclusionPath {
            value, auth_path, ..
        } = self;

        let root = Root::from_allocated::<CS>(root.clone());
        PoRCircuit::<Bls12, H>::synthesize(cs, params, value, auth_path, root, true)
    }
}

impl<H: Hasher> From<MerkleProof<H>> for InclusionPath<H> {
    fn from(other: MerkleProof<H>) -> Self {
        let (value, auth_path) = other.into_options_with_leaf();

        InclusionPath {
            value,
            auth_path,
            _h: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplicaColumnProof<H: Hasher> {
    c_x: ColumnProof<H>,
    c_inv_x: ColumnProof<H>,
    drg_parents: Vec<ColumnProof<H>>,
    exp_parents_even: Vec<ColumnProof<H>>,
    exp_parents_odd: Vec<ColumnProof<H>>,
}

impl<H: Hasher> ReplicaColumnProof<H> {
    /// Create an empty proof, used in `blank_circuit`s.
    pub fn empty(params: &PublicParams<H>) -> Self {
        ReplicaColumnProof {
            c_x: ColumnProof::empty_all(params),
            c_inv_x: ColumnProof::empty_all(params),
            drg_parents: vec![ColumnProof::empty_all(params); params.graph.base_graph().degree()],
            exp_parents_even: vec![
                ColumnProof::empty_even(params);
                params.graph.expansion_degree()
            ],
            exp_parents_odd: vec![ColumnProof::empty_odd(params); params.graph.expansion_degree()],
        }
    }
}

impl<H: Hasher> From<VanillaReplicaColumnProof<H>> for ReplicaColumnProof<H> {
    fn from(vanilla_proof: VanillaReplicaColumnProof<H>) -> Self {
        let VanillaReplicaColumnProof {
            c_x,
            c_inv_x,
            drg_parents,
            exp_parents_even,
            exp_parents_odd,
        } = vanilla_proof;

        ReplicaColumnProof {
            c_x: c_x.into(),
            c_inv_x: c_inv_x.into(),
            drg_parents: drg_parents.into_iter().map(|p| p.into()).collect(),
            exp_parents_even: exp_parents_even.into_iter().map(|p| p.into()).collect(),
            exp_parents_odd: exp_parents_odd.into_iter().map(|p| p.into()).collect(),
        }
    }
}
