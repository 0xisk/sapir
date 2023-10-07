use crate::frontend::constraint_system::Wire;
use ark_ff::BigInteger;
use ark_ff::PrimeField;

pub fn xor_64<F: PrimeField>(a: [Wire<F>; 64], b: [Wire<F>; 64]) -> [Wire<F>; 64] {
    let cs = a[0].cs();
    assert_eq!(a.len(), b.len());
    let mut out = [cs.one(); 64];
    for i in 0..64 {
        out[i] = bit_xor(a[i], b[i]);
    }

    out
}

// (!a) & b
pub fn not_a_and_b<F: PrimeField>(a: Wire<F>, b: Wire<F>) -> Wire<F> {
    let cs = a.cs();

    let one = cs.one();
    // (a * -1 + 1 * 1) * (b * 1) = c
    cs.constrain(&[(a, -F::ONE), (one, F::ONE)], &[(b, F::ONE)], &[])
}

pub fn not_a_and_b_64<F: PrimeField>(a: [Wire<F>; 64], b: [Wire<F>; 64]) -> [Wire<F>; 64] {
    let cs = a[0].cs();
    assert_eq!(a.len(), b.len());
    let mut out = [cs.one(); 64];
    for i in 0..64 {
        out[i] = not_a_and_b(a[i], b[i]);
    }

    out
}

pub fn rotate_left_64<F: PrimeField>(a: [Wire<F>; 64], n: usize) -> [Wire<F>; 64] {
    let mut out = Vec::with_capacity(64);
    for i in 0..64 {
        out.push(a[((i as usize).wrapping_sub(n)) % 64]);
    }

    out.try_into().unwrap()
}

pub fn bit_xor<F: PrimeField>(a: Wire<F>, b: Wire<F>) -> Wire<F> {
    let cs = a.cs();

    // -2a * b + a + b = c
    cs.constrain(
        &[(a, -F::from(2u32))],
        &[(b, F::ONE)],
        &[(a, F::ONE), (b, F::ONE)],
    )
}

// Interprets the bits as LSB first.
pub fn from_bits<F: PrimeField>(bits: &[Wire<F>]) -> Wire<F> {
    let cs = bits[0].cs();
    let mut terms = Vec::with_capacity(64);

    let mut pow = F::from(1u32);
    for bit in bits.iter().rev() {
        terms.push((cs.mul_const(*bit, pow), true));
        pow *= F::from(2u32);
    }

    cs.sum(&terms)
}

pub fn to_bits<F: PrimeField>(a: Wire<F>, field_bits: usize) -> Vec<Wire<F>> {
    let cs = a.cs();

    let bits = (0..field_bits)
        .map(|_| cs.alloc_var(F::ZERO))
        .collect::<Vec<Wire<F>>>();

    if cs.is_witness_gen() {
        let a_assigned = cs.wires[a.index];
        let a_bytes = a_assigned.into_bigint().to_bytes_be();
        for (i, b) in a_bytes.iter().enumerate() {
            // Little-endian bits
            for j in 0..8 {
                let bit = if (*b >> j) & 1 == 1 { F::ONE } else { F::ZERO };
                cs.wires[bits[i * 8 + j].index] = bit;
            }
        }
    }

    let mut sum = cs.zero();

    let mut pow = F::ONE;
    for (i, byte) in bits.chunks(8).rev().enumerate() {
        let mut byte = byte.to_vec();
        byte.reverse();
        let term = from_bits(&byte);
        sum += cs.mul_const(term, pow);

        pow = F::from(2u32).pow(&[8 * ((i + 1) as u64)]);
    }

    cs.assert_equal(a, sum, "to_bits failed");

    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::constraint_system::ConstraintSystem;
    use ark_ff::{Field, PrimeField};

    type Fp = ark_secq256k1::Fr;

    #[test]
    pub fn test_from_bits() {
        let synthesizer = |cs: &mut ConstraintSystem<Fp>| {
            let bits = cs.alloc_priv_inputs(256);
            let out = from_bits(&bits);

            cs.expose_public(out);
        };

        let mut bits = vec![Fp::ZERO; 256];
        bits[253] = Fp::ONE;
        let expected = Fp::from(4);

        let priv_input = bits;
        let pub_input = vec![expected];

        let mut cs = ConstraintSystem::new();
        let witness = cs.gen_witness(synthesizer, &pub_input, &priv_input);

        cs.set_constraints(&synthesizer);
        assert!(cs.is_sat(&witness, &pub_input));
    }

    #[test]
    fn test_to_bits() {
        let synthesizer = |cs: &mut ConstraintSystem<Fp>| {
            let val = cs.alloc_priv_input();
            let out = to_bits(val, 256);

            for out_i in out {
                cs.expose_public(out_i);
            }
        };

        let val = Fp::from(123);
        let expected_bits = val
            .into_bigint()
            .to_bits_be()
            .iter()
            .map(|b| Fp::from(*b))
            .collect::<Vec<Fp>>();

        let priv_input = [val];
        let pub_input = expected_bits;

        let mut cs = ConstraintSystem::new();
        let witness = cs.gen_witness(synthesizer, &pub_input, &priv_input);

        cs.set_constraints(&synthesizer);
        assert!(cs.is_sat(&witness, &pub_input));
    }
}
