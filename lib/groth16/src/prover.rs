use crate::{r1cs_to_qap::R1CStoQAP, Proof, ProvingKey, VerifyingKey};
use ark_ec::{msm::VariableBaseMSM, AffineCurve, PairingEngine, ProjectiveCurve};
use ark_ff::{Field, PrimeField, UniformRand, Zero};
use ark_poly::GeneralEvaluationDomain;
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef,
    OptimizationGoal, Result as R1CSResult,
};
use ark_std::{cfg_into_iter, cfg_iter, rand::Rng, vec::Vec};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Create a Groth16 proof that is zero-knowledge.
/// This method samples randomness for zero knowledges via `rng`.
#[inline]
pub fn create_random_proof<E, C, R>(
    circuit: C,
    pk: &ProvingKey<E>,
    rng: &mut R,
) -> R1CSResult<Proof<E>>
where
    E: PairingEngine,
    C: ConstraintSynthesizer<E::Fr>,
    R: Rng,
{
    let gc = start_timer!(|| "Before Prove");

    let r = E::Fr::rand(rng);
    let s = E::Fr::rand(rng);

    // create_proof::<E, C>(circuit, pk, r, s)

    let cs = ConstraintSystem::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    circuit.generate_constraints(cs.clone())?;
    cs.finalize();
    let matrices = cs.to_matrices().unwrap();
    end_timer!(gc);
    // println!("{}", cs.borrow_mut().unwrap().witness_assignment.len());
    // let mut idx = 0;
    // for i in cs.borrow_mut().unwrap().witness_assignment.iter_mut() {
    //     if idx < 3 * 4096 {
    //         *i = E::Fr::from(0xffffee001u64)
    //             * E::Fr::from(0xffffc4001u64)
    //             * E::Fr::from(0x1ffffe0001u64);
    //     } else if idx < 5 * 4096 {
    //         *i = E::Fr::from(19u64);
    //     } else if idx < 6 * 4096 {
    //         *i = E::Fr::from(256u64);
    //     } else {
    //         *i = E::Fr::from(0x1u64);
    //     }
    // }
    // cs.borrow_mut().unwrap().witness_assignment[0] = E::Fr::rand(rng);
    // println!("{}", cs.borrow_mut().unwrap().witness_assignment[0]);
    lonhh_create_proof::<E, C>(cs, &matrices, pk, r, s)
}

/// Create a Groth16 proof that is *not* zero-knowledge.
#[inline]
pub fn create_proof_no_zk<E, C>(circuit: C, pk: &ProvingKey<E>) -> R1CSResult<Proof<E>>
where
    E: PairingEngine,
    C: ConstraintSynthesizer<E::Fr>,
{
    create_proof::<E, C>(circuit, pk, E::Fr::zero(), E::Fr::zero())
}

/// Create a Groth16 proof using randomness `r` and `s`.
#[inline]
pub fn create_proof<E, C>(
    circuit: C,
    pk: &ProvingKey<E>,
    r: E::Fr,
    s: E::Fr,
) -> R1CSResult<Proof<E>>
where
    E: PairingEngine,
    C: ConstraintSynthesizer<E::Fr>,
{
    type D<F> = GeneralEvaluationDomain<F>;

    let prover_time = start_timer!(|| "Groth16::Prover");
    let cs = ConstraintSystem::new_ref();

    // Set the optimization goal
    cs.set_optimization_goal(OptimizationGoal::Constraints);

    // Synthesize the circuit.
    // let synthesis_time = start_timer!(|| "Constraint synthesis");
    let gc = start_timer!(|| "Generate constraints");
    circuit.generate_constraints(cs.clone())?;
    end_timer!(gc);
    debug_assert!(cs.is_satisfied().unwrap());
    // end_timer!(synthesis_time);

    let lc_time = start_timer!(|| "Inlining LCs");
    cs.finalize();
    end_timer!(lc_time);

    let witness_map_time = start_timer!(|| "R1CS to QAP witness map");
    let h = R1CStoQAP::witness_map::<E::Fr, D<E::Fr>>(cs.clone())?;
    end_timer!(witness_map_time);
    let h_assignment = cfg_into_iter!(h).map(|s| s.into()).collect::<Vec<_>>();
    let c_acc_time = start_timer!(|| "Compute C");

    let h_acc = VariableBaseMSM::multi_scalar_mul(&pk.h_query, &h_assignment);
    drop(h_assignment);
    // Compute C
    let prover = cs.borrow().unwrap();
    let aux_assignment = cfg_iter!(prover.witness_assignment)
        .map(|s| s.into_repr())
        .collect::<Vec<_>>();

    let l_aux_acc = VariableBaseMSM::multi_scalar_mul(&pk.l_query, &aux_assignment);

    let r_s_delta_g1 = pk.delta_g1.into_projective().mul(r.into()).mul(s.into());

    end_timer!(c_acc_time);

    let input_assignment = prover.instance_assignment[1..]
        .iter()
        .map(|s| s.into_repr())
        .collect::<Vec<_>>();

    drop(prover);
    drop(cs);

    let assignment = [&input_assignment[..], &aux_assignment[..]].concat();
    drop(aux_assignment);

    // Compute A
    let a_acc_time = start_timer!(|| "Compute A");
    let r_g1 = pk.delta_g1.mul(r);

    let g_a = calculate_coeff(r_g1, &pk.a_query, pk.vk.alpha_g1, &assignment);

    let s_g_a = g_a.mul(s.into());
    end_timer!(a_acc_time);

    // Compute B in G1 if needed
    let g1_b = if !r.is_zero() {
        let b_g1_acc_time = start_timer!(|| "Compute B in G1");
        let s_g1 = pk.delta_g1.mul(s);
        let g1_b = calculate_coeff(s_g1, &pk.b_g1_query, pk.beta_g1, &assignment);

        end_timer!(b_g1_acc_time);

        g1_b
    } else {
        E::G1Projective::zero()
    };

    // Compute B in G2
    let b_g2_acc_time = start_timer!(|| "Compute B in G2");
    let s_g2 = pk.vk.delta_g2.mul(s);
    let g2_b = calculate_coeff(s_g2, &pk.b_g2_query, pk.vk.beta_g2, &assignment);
    let r_g1_b = g1_b.mul(r.into());
    drop(assignment);

    end_timer!(b_g2_acc_time);

    let c_time = start_timer!(|| "Finish C");
    let mut g_c = s_g_a;
    g_c += &r_g1_b;
    g_c -= &r_s_delta_g1;
    g_c += &l_aux_acc;
    g_c += &h_acc;
    end_timer!(c_time);

    end_timer!(prover_time);

    Ok(Proof {
        a: g_a.into_affine(),
        b: g2_b.into_affine(),
        c: g_c.into_affine(),
    })
}

/// Modified proof generation algorithm, used for batch proof generation
#[inline]
pub fn lonhh_create_proof<E, C>(
    cs: ConstraintSystemRef<<E as PairingEngine>::Fr>,
    matrices: &ConstraintMatrices<E::Fr>,
    pk: &ProvingKey<E>,
    r: E::Fr,
    s: E::Fr,
) -> R1CSResult<Proof<E>>
where
    E: PairingEngine,
    C: ConstraintSynthesizer<E::Fr>,
{
    type D<F> = GeneralEvaluationDomain<F>;

    let prover_time = start_timer!(|| "Groth16::Prover");

    debug_assert!(cs.is_satisfied().unwrap());

    let witness_map_time = start_timer!(|| "R1CS to QAP witness map");
    let h = R1CStoQAP::lonhh_witness_map::<E::Fr, D<E::Fr>>(cs.clone(), &matrices)?;
    let h_assignment = cfg_into_iter!(h).map(|s| s.into()).collect::<Vec<_>>();
    end_timer!(witness_map_time);
    let c_acc_time = start_timer!(|| "Compute C");

    let c_acc_time1 = start_timer!(|| "Compute C1");
    let h_acc = VariableBaseMSM::multi_scalar_mul(&pk.h_query, &h_assignment);
    end_timer!(c_acc_time1);
    drop(h_assignment);
    // Compute C
    let c_acc_time2 = start_timer!(|| "Compute C2");
    let prover = cs.borrow().unwrap();
    let aux_assignment = cfg_iter!(prover.witness_assignment)
        .map(|s| s.into_repr())
        .collect::<Vec<_>>();
    end_timer!(c_acc_time2);

    let l_aux_acc = VariableBaseMSM::multi_scalar_mul(&pk.l_query, &aux_assignment);

    let r_s_delta_g1 = pk.delta_g1.into_projective().mul(r.into()).mul(s.into());

    end_timer!(c_acc_time);

    let input_assignment = prover.instance_assignment[1..]
        .iter()
        .map(|s| s.into_repr())
        .collect::<Vec<_>>();

    drop(prover);
    drop(cs);

    let assignment = [&input_assignment[..], &aux_assignment[..]].concat();
    drop(aux_assignment);

    // Compute A
    let a_acc_time = start_timer!(|| "Compute A");
    let r_g1 = pk.delta_g1.mul(r);

    let g_a = calculate_coeff(r_g1, &pk.a_query, pk.vk.alpha_g1, &assignment);

    let s_g_a = g_a.mul(s.into());
    end_timer!(a_acc_time);

    // Compute B in G1 if needed
    let g1_b = if !r.is_zero() {
        let b_g1_acc_time = start_timer!(|| "Compute B in G1");
        let s_g1 = pk.delta_g1.mul(s);
        let g1_b = calculate_coeff(s_g1, &pk.b_g1_query, pk.beta_g1, &assignment);

        end_timer!(b_g1_acc_time);

        g1_b
    } else {
        E::G1Projective::zero()
    };

    // Compute B in G2
    let b_g2_acc_time = start_timer!(|| "Compute B in G2");
    let s_g2 = pk.vk.delta_g2.mul(s);
    let g2_b = calculate_coeff(s_g2, &pk.b_g2_query, pk.vk.beta_g2, &assignment);
    let r_g1_b = g1_b.mul(r.into());
    drop(assignment);

    end_timer!(b_g2_acc_time);

    let c_time = start_timer!(|| "Finish C");
    let mut g_c = s_g_a;
    g_c += &r_g1_b;
    g_c -= &r_s_delta_g1;
    g_c += &l_aux_acc;
    g_c += &h_acc;
    end_timer!(c_time);

    end_timer!(prover_time);

    Ok(Proof {
        a: g_a.into_affine(),
        b: g2_b.into_affine(),
        c: g_c.into_affine(),
    })
}

/// Given a Groth16 proof, returns a fresh proof of the same statement. For a

/// Given a Groth16 proof, returns a fresh proof of the same statement. For a
/// proof π of a statement S, the output of the non-deterministic procedure
/// `rerandomize_proof(π)` is statistically indistinguishable from a fresh
/// honest proof of S. For more info, see theorem 3 of [\[BKSV20\]](https://eprint.iacr.org/2020/811)
pub fn rerandomize_proof<E, R>(rng: &mut R, vk: &VerifyingKey<E>, proof: &Proof<E>) -> Proof<E>
where
    E: PairingEngine,
    R: Rng,
{
    // These are our rerandomization factors. They must be nonzero and uniformly
    // sampled.
    let (mut r1, mut r2) = (E::Fr::zero(), E::Fr::zero());
    while r1.is_zero() || r2.is_zero() {
        r1 = E::Fr::rand(rng);
        r2 = E::Fr::rand(rng);
    }

    // See figure 1 in the paper referenced above:
    //   A' = (1/r₁)A
    //   B' = r₁B + r₁r₂(δG₂)
    //   C' = C + r₂A

    // We can unwrap() this because r₁ is guaranteed to be nonzero
    let new_a = proof.a.mul(r1.inverse().unwrap());
    let new_b = proof.b.mul(r1) + &vk.delta_g2.mul(r1 * &r2);
    let new_c = proof.c + proof.a.mul(r2).into_affine();

    Proof {
        a: new_a.into_affine(),
        b: new_b.into_affine(),
        c: new_c,
    }
}

fn calculate_coeff<G: AffineCurve>(
    initial: G::Projective,
    query: &[G],
    vk_param: G,
    assignment: &[<G::ScalarField as PrimeField>::BigInt],
) -> G::Projective {
    let el = query[0];
    let acc = VariableBaseMSM::multi_scalar_mul(&query[1..], assignment);

    let mut res = initial;
    res.add_assign_mixed(&el);
    res += &acc;
    res.add_assign_mixed(&vk_param);

    res
}
