//! This module implements Plonk prover polynomial evaluations primitive.

use ark_ff::Field;
use oracle::sponge::ScalarChallenge;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use super::{polynomial::COLUMNS, wires::PERMUTS};

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
pub enum Evaluation<F> {
    #[serde_as(as = "Vec<o1_utils::serialization::SerdeAs>")]
    Chunked(Vec<F>),
    #[serde_as(as = "o1_utils::serialization::SerdeAs")]
    NotChunked(F),
}

#[derive(Clone, Debug)]
pub struct RandomOracles<F: Field> {
    pub joint_combiner: (ScalarChallenge<F>, F),
    pub beta: F,
    pub gamma: F,
    pub alpha_chal: ScalarChallenge<F>,
    pub alpha: F,
    pub zeta: F,
    pub v: F,
    pub u: F,
    pub zeta_chal: ScalarChallenge<F>,
    pub v_chal: ScalarChallenge<F>,
    pub u_chal: ScalarChallenge<F>,
}

impl<F: Field> Default for RandomOracles<F> {
    fn default() -> Self {
        let c = ScalarChallenge(F::zero());
        Self {
            beta: F::zero(),
            gamma: F::zero(),
            alpha: F::zero(),
            zeta: F::zero(),
            v: F::zero(),
            u: F::zero(),
            alpha_chal: c,
            zeta_chal: c,
            v_chal: c,
            u_chal: c,
            joint_combiner: (c, F::zero()),
        }
    }
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
pub struct LookupChunkedEvaluations<F> {
    #[serde_as(as = "Vec<Vec<o1_utils::serialization::SerdeAs>>")]
    /// sorted lookup table polynomial
    pub sorted: Vec<Vec<F>>,
    /// lookup aggregation polynomial
    #[serde_as(as = "Vec<o1_utils::serialization::SerdeAs>")]
    pub aggreg: Vec<F>,
    // TODO: May be possible to optimize this away?
    /// lookup table polynomial
    #[serde_as(as = "Vec<o1_utils::serialization::SerdeAs>")]
    pub table: Vec<F>,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
pub struct ProofChunkedEvaluations<F> {
    /// witness polynomials
    #[serde_as(as = "[Vec<o1_utils::serialization::SerdeAs>; COLUMNS]")]
    pub w: [Vec<F>; COLUMNS],
    /// permutation polynomial
    #[serde_as(as = "Vec<o1_utils::serialization::SerdeAs>")]
    pub z: Vec<F>,
    /// permutation polynomials
    /// (PERMUTS-1 evaluations because the last permutation is only used in commitment form)
    #[serde_as(as = "[Vec<o1_utils::serialization::SerdeAs>; PERMUTS - 1]")]
    pub s: [Vec<F>; PERMUTS - 1],
    /// lookup-related evaluations
    #[serde(bound = "LookupChunkedEvaluations<F>: Serialize + DeserializeOwned")]
    pub lookup: Option<LookupChunkedEvaluations<F>>,
    /// evaluation of the generic selector polynomial
    #[serde_as(as = "Vec<o1_utils::serialization::SerdeAs>")]
    pub generic_selector: Vec<F>,
    /// evaluation of the poseidon selector polynomial
    #[serde_as(as = "Vec<o1_utils::serialization::SerdeAs>")]
    pub poseidon_selector: Vec<F>,
}

//
// OCaml types
//

#[cfg(feature = "ocaml_types")]
pub mod caml {
    use super::*;
    use oracle::sponge::caml::CamlScalarChallenge;

    //
    // RandomOracles<F> <-> CamlRandomOracles<CamlF>
    //

    #[derive(ocaml::IntoValue, ocaml::FromValue, ocaml_gen::Struct)]
    pub struct CamlRandomOracles<CamlF> {
        pub joint_combiner: (CamlScalarChallenge<CamlF>, CamlF),
        pub beta: CamlF,
        pub gamma: CamlF,
        pub alpha_chal: CamlScalarChallenge<CamlF>,
        pub alpha: CamlF,
        pub zeta: CamlF,
        pub v: CamlF,
        pub u: CamlF,
        pub zeta_chal: CamlScalarChallenge<CamlF>,
        pub v_chal: CamlScalarChallenge<CamlF>,
        pub u_chal: CamlScalarChallenge<CamlF>,
    }

    impl<F, CamlF> From<RandomOracles<F>> for CamlRandomOracles<CamlF>
    where
        F: Field,
        CamlF: From<F>,
    {
        fn from(ro: RandomOracles<F>) -> Self {
            Self {
                joint_combiner: (ro.joint_combiner.0.into(), ro.joint_combiner.1.into()),
                beta: ro.beta.into(),
                gamma: ro.gamma.into(),
                alpha_chal: ro.alpha_chal.into(),
                alpha: ro.alpha.into(),
                zeta: ro.zeta.into(),
                v: ro.v.into(),
                u: ro.u.into(),
                zeta_chal: ro.zeta_chal.into(),
                v_chal: ro.v_chal.into(),
                u_chal: ro.u_chal.into(),
            }
        }
    }

    impl<F, CamlF> Into<RandomOracles<F>> for CamlRandomOracles<CamlF>
    where
        CamlF: Into<F>,
        F: Field,
    {
        fn into(self) -> RandomOracles<F> {
            RandomOracles {
                joint_combiner: (self.joint_combiner.0.into(), self.joint_combiner.1.into()),
                beta: self.beta.into(),
                gamma: self.gamma.into(),
                alpha_chal: self.alpha_chal.into(),
                alpha: self.alpha.into(),
                zeta: self.zeta.into(),
                v: self.v.into(),
                u: self.u.into(),
                zeta_chal: self.zeta_chal.into(),
                v_chal: self.v_chal.into(),
                u_chal: self.u_chal.into(),
            }
        }
    }
}
