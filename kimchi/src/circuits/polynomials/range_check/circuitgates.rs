///```text
/// Range check circuit gates:
///
///    The range check gate is comprised of three circuit gates (RangeCheck0, RangeCheck1
///    and Zero) and can perform range checks on up to three 88-bit values: v0, v1 and v2.
///
///    Byte-order:
///      * Each value is in little-endian byte order
///      * Limbs are mapped to columns in big-endian order (i.e. the lowest columns
///        contain the highest bits)
///      * We also have the highest bits covered by copy constraints and plookups, so that
///        we can copy the highest two constraints to zero and get a 64-bit lookup, which
///        are envisioned as a common case
///
///    The values are decomposed into limbs as follows.
///
///    L is a 12-bit lookup (or copy) limb,
///    C is a 2-bit "crumb" limb.
///
///         <----6----> <------8------>
///    v0 = L L L L L L C C C C C C C C
///    v1 = L L L L L L C C C C C C C C
///         <--4--> <------------------20----------------->
///    v2 = L L L L C C C C C C C C C C C C C C C C C C C C
///
/// Witness structure:
///
///   Row  Contents
///     0   v0
///     1   v1
///     2   v2
///     3   v0,v1,v2
///
///   * The first 2 rows contain v0 and v1 and their respective decompositions
///     into 12-bit and 2-bit limbs
///   * The 3rd row contains v2 and part of its decomposition: four 12-bit limbs and
///     the 1st 10 crumbs
///   * The final row contains v0's and v1's 5th and 6th 12-bit limbs as well as the
///     remaining 10 crumbs of v2
///
/// Constraints:
///
///   For efficiency, the values are constrained differently according to their type.
///    * 12-bit limbs are constrained with plookups
///    * 2-bit crumbs are constrained with degree-4 constraints
///
/// Layout:
///
///  This is how three 88-bit inputs v0, v1 and v2 are layed out and constrained.
///
///   * vipj is the jth 12-bit limb of value vi
///   * vicj is the jth 2-bit crumb limb of value vi
///
/// Gate:   RangeCheck0    RangeCheck0    RangeCheck1    Zero
///    Rows -->
///         0              1              2              3
///    0 | v0           | v1           | v2           | 0
///    1 | copy    v0p0 | copy    v1p0 | crumb   v2c0 | crumb   v2c10 <- MSB
///    2 | copy    v0p1 | copy    v1p1 | crumb   v2c1 | crumb   v2c11
///    3 | plookup v0p2 | plookup v1p2 | plookup v2p0 | plookup  v0p0
///    4 | plookup v0p3 | plookup v1p3 | plookup v2p1 | plookup  v0p1
///    5 | plookup v0p4 | plookup v1p4 | plookup v2p2 | plookup  v1p0
///    6 | plookup v0p5 | plookup v1p5 | plookup v2p3 | plookup  v1p1
///    7 | crumb   v0c0 | crumb   v1c0 | crumb   v2c2 | crumb   v2c12
///    8 | crumb   v0c1 | crumb   v1c1 | crumb   v2c3 | crumb   v2c13
///    9 | crumb   v0c2 | crumb   v1c2 | crumb   v2c4 | crumb   v2c14
///   10 | crumb   v0c3 | crumb   v1c3 | crumb   v2c5 | crumb   v2c15
///   11 | crumb   v0c4 | crumb   v1c4 | crumb   v2c6 | crumb   v2c16
///   12 | crumb   v0c5 | crumb   v1c5 | crumb   v2c7 | crumb   v2c17
///   13 | crumb   v0p6 | copy    v1c6 | crumb   v2c8 | crumb   v2c18
///   14 | crumb   v0p7 | copy    v1c7 | crumb   v2c9 | crumb   v2c19 <- LSB
///
///   The 12-bit chunks are constrained with plookups and the 2-bit crumbs
///   constrained with degree-4 constraints of the form x*(x - 1)*(x - 2)*(x - 3).
///
///   Note that copy denotes a plookup that is deferred to the 4th gate (i.e. Zero).
///   This is because of the limitation that we have at most 4 lookups per row.
///   The copies are constrained using the permutation argument.
///
/// Gate types:
///
///   Different rows are constrained using different CircuitGate types
///
///   Row   CircuitGate   Purpose
///     0   RangeCheck0   Partially constrain v0
///     1   RangeCheck0   Partially constrain v1
///     2   RangeCheck1   Fully constrain v2 (and trigger plookups constraints on row 3)
///     3   Zero          Complete the constraining of v0 and v1 using lookups
///
///  Nb. each CircuitGate type corresponds to a unique polynomial and thus
///       is assigned its own unique powers of alpha
///```
use std::marker::PhantomData;

use crate::circuits::{
    argument::{Argument, ArgumentType},
    expr::{constraints::crumb, witness_curr, witness_next, E},
    gate::GateType,
    polynomial::COLUMNS,
};
use ark_ff::{FftField, One, Zero};

/// RangeCheck0 - Range check constraints
///
///   * This circuit gate is used to partially constrain values v0 and v1
///   * The rest of v0 and v1 are constrained by the lookups in the Zero gate row
///   * This gate operates on the Curr row
///
/// It uses three different types of constraints
///   * copy    - copy to another cell (12-bits)
///   * plookup - plookup (12-bits)
///   * crumb   - degree-4 constraint (2-bits)
///
/// Given value v the layout looks like this
///
/// Column | Curr
///      0 | v
///      1 | copy    vp0
///      2 | copy    vp1
///      3 | plookup vp2
///      4 | plookup vp3
///      5 | plookup vp4
///      6 | plookup vp5
///      7 | crumb   vc0
///      8 | crumb   vc1
///      9 | crumb   vc2
///     10 | crumb   vc3
///     11 | crumb   vc4
///     12 | crumb   vc5
///     13 | crumb   vc6
///     14 | crumb   vc7
///
/// where the notation vpi and vci defined in the "Layout" section above.

#[derive(Default)]
pub struct RangeCheck0<F>(PhantomData<F>);

impl<F> Argument<F> for RangeCheck0<F>
where
    F: FftField,
{
    const ARGUMENT_TYPE: ArgumentType = ArgumentType::Gate(GateType::RangeCheck0);
    const CONSTRAINTS: u32 = 9;

    // Constraints for RangeCheck0
    //   * Operates on Curr row
    //   * Range constrain all limbs except vp0 and vp1 (barring plookup constraints, which are done elsewhere)
    //   * Constrain that combining all limbs equals the limb stored in column 0
    fn constraints() -> Vec<E<F>> {
        // 1) Apply range constraints on limbs
        //    * Columns 1-2 are 12-bit copy constraints
        //        * They are copied to 3 rows ahead and are constrained by plookups
        //          triggered by RangeCheck1 on its Next row
        //        * They can be constrained to zero to obtain a 64-bit range check
        //    * Columns 3-6 are 12-bit plookup range constraints (these are specified in the lookup gate)
        //    * Columns 7-14 are 2-bit crumb range constraints
        let mut constraints = (7..COLUMNS)
            .map(|i| crumb(&witness_curr(i)))
            .collect::<Vec<E<F>>>();

        // 2) Constrain that the combined limbs equals the value v stored in w(0):
        //
        //        w(0) = v = vp0 vp1 vp2 vp3 vp4 vp5 vc0 vc1 vc2 vc3 vc4 vc5 vc6 vc7
        //
        //    where the value and limbs are stored in little-endian byte order, but mapped
        //    to cells in big-endian order.
        //
        //    Cols: 0  1   2   3   4   5   6   7   8   9   10  11  12  13  14
        //    Curr: v  vp0 vp1 vp2 vp3 vp4 vp5 vc0 vc1 vc2 vc3 vc4 vc5 vc6 vc7  <- LSB

        let mut power_of_2 = E::one();
        let mut sum_of_limbs = E::zero();

        // Sum 2-bit limbs
        for i in (7..COLUMNS).rev() {
            sum_of_limbs += power_of_2.clone() * witness_curr(i);
            power_of_2 *= 4u64.into(); // 2 bits
        }

        // Sum 12-bit limbs
        for i in (1..=6).rev() {
            sum_of_limbs += power_of_2.clone() * witness_curr(i);
            power_of_2 *= 4096u64.into(); // 12 bits
        }

        // Check value v against the sum of limbs
        constraints.push(sum_of_limbs - witness_curr(0));

        constraints
    }
}

/// RangeCheck1 - Range check constraints
///
///   * This circuit gate is used to fully constrain v2
///   * It operates on the Curr and Next rows
///
/// It uses two different types of constraints
///   * plookup - plookup (12-bits)
///   * crumb   - degree-4 constraint (2-bits)
///
/// Given value v2 the layout looks like this
///
/// Column | Curr         | Next
///      0 | v2           | (ignored)
///      1 | crumb   v2c0 | crumb v2c10
///      2 | crumb   v2c1 | crumb v2c11
///      3 | plookup v2p0 | (ignored)
///      4 | plookup v2p1 | (ignored)
///      5 | plookup v2p2 | (ignored)
///      6 | plookup v2p3 | (ignored)
///      7 | crumb   v2c2 | crumb v2c12
///      8 | crumb   v2c3 | crumb v2c13
///      9 | crumb   v2c4 | crumb v2c14
///     10 | crumb   v2c5 | crumb v2c15
///     11 | crumb   v2c6 | crumb v2c16
///     12 | crumb   v2c7 | crumb v2c17
///     13 | crumb   v2c8 | crumb v2c18
///     14 | crumb   v2c9 | crumb v2c19
///
/// where the notation v2i and v2i defined in the "Layout" section above.

#[derive(Default)]
pub struct RangeCheck1<F>(PhantomData<F>);

impl<F> Argument<F> for RangeCheck1<F>
where
    F: FftField,
{
    const ARGUMENT_TYPE: ArgumentType = ArgumentType::Gate(GateType::RangeCheck1);
    const CONSTRAINTS: u32 = 21;

    // Constraints for RangeCheck1
    //   * Operates on Curr and Next row
    //   * Range constrain all limbs (barring plookup constraints, which are done elsewhere)
    //   * Constrain that combining all limbs equals the value v2 stored in row Curr, column 0
    fn constraints() -> Vec<E<F>> {
        // 1) Apply range constraints on limbs for Curr row
        //    * Columns 1-2 are 2-bit crumbs
        let mut constraints = (1..=2)
            .map(|i| crumb(&witness_curr(i)))
            .collect::<Vec<E<F>>>();
        //    * Columns 3-6 are 12-bit plookup range constraints (these are specified
        //      in the lookup gate)
        //    * Columns 7-14 are 2-bit crumb range constraints
        constraints.append(
            &mut (7..COLUMNS)
                .map(|i| crumb(&witness_curr(i)))
                .collect::<Vec<E<F>>>(),
        );

        // 2) Apply range constraints on limbs for Next row
        //    * Columns 1-2 are 2-bit crumbs
        constraints.append(
            &mut (1..=2)
                .map(|i| crumb(&witness_next(i)))
                .collect::<Vec<E<F>>>(),
        );
        //    * Columns 3-6 are 12-bit plookup range constraints for v0 and v1 (these
        //      are specified in the lookup gate)
        //    * Columns 7-14 are more 2-bit crumbs
        constraints.append(
            &mut (7..COLUMNS)
                .map(|i| crumb(&witness_next(i)))
                .collect::<Vec<E<F>>>(),
        );

        // 2) Constrain that the combined limbs equals the value v2 stored in w(0) where
        //
        //    w(0) = v2 = vc0 vc1 vp0 vp1 vp2 vp3 vc2 vc3 vc4 vc5 vc6 vc7 vc8 vc9 vc10 vc11 vc12
        //                vc13 vc14 vc15 vc16 vc17 vc18 vc19
        //
        //    where the value and limbs are stored in little-endian byte order, but mapped
        //    to cells in big-endian order.
        //
        //          0  1   2   3   4   5    6    7    8    9    10   11   12   13   14
        //    Curr  v2 vc0 vc1 vp0 vp1 vp2  vp3  vc2  vc3  vc4  vc5  vc6  vc7  vc8  vc9
        //    Next                     vc10 vc11 vc12 vc13 vc14 vc15 vc16 vc17 vc18 vc19 <- LSB

        let mut power_of_2 = E::one();
        let mut sum_of_limbs = E::zero();

        // Next row: Sum 2-bit limbs
        for i in (7..COLUMNS).rev() {
            sum_of_limbs += power_of_2.clone() * witness_next(i);
            power_of_2 *= 4u64.into(); // 2 bits
        }

        // Next row:  Sum remaining 2-bit limbs vc10 and vc11
        for i in (1..=2).rev() {
            sum_of_limbs += power_of_2.clone() * witness_next(i);
            power_of_2 *= 4u64.into(); // 2 bits
        }

        // Curr row:  Sum 2-bit limbs
        for i in (7..COLUMNS).rev() {
            sum_of_limbs += power_of_2.clone() * witness_curr(i);
            power_of_2 *= 4u64.into(); // 2 bits
        }

        // Curr row: Sum 12-bit limbs
        for i in (3..=6).rev() {
            sum_of_limbs += power_of_2.clone() * witness_curr(i);
            power_of_2 *= 4096u64.into(); // 12 bits
        }

        // Curr row:  Sum remaining 2-bit limbs: vc0 and vc1
        for i in (1..=2).rev() {
            sum_of_limbs += power_of_2.clone() * witness_curr(i);
            power_of_2 *= 4u64.into(); // 2 bits
        }

        // Check value v2 against the sum of limbs
        constraints.push(sum_of_limbs - witness_curr(0));

        constraints
    }
}
