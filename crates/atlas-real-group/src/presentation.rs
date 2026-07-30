//! The display-facing presentation of one real form: its Lie-algebra name
//! plus the topology status bits that `real_form_value::print` reports
//! (interpreter/atlas-types.w:3566-3575).
//!
//! The bits are the upstream `RealReductiveGroup::construct` status flags
//! (realredgp.cpp:68-80): `IsCompact`/`IsSplit` compare the most split
//! Cartan involution `ms_tau` against `+/-1`, `IsQuasisplit` compares the
//! form number against the quasisplit form, and `IsConnected` is the
//! triviality of the topology module's dual component group.

use crate::cartan_classification::CartanClassification;
use crate::error::StructureError;
use crate::form_name::form_type_name;
use crate::inner_class::InnerClass;
use crate::integer_lattice::IntegerLatticeBudget;
use crate::layout::InnerClassLayout;
use crate::real_form_order::ExternalFormOrder;

/// One real form's display data, in external (interface) numbering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealFormPresentation {
    /// The `printType` Lie-algebra name.
    pub name: String,
    /// `IsCompact`: the most split Cartan involution is the identity.
    pub compact: bool,
    /// `IsConnected`: the dual component group is trivial.
    pub connected: bool,
    /// `IsSplit`: the most split Cartan involution is minus the identity.
    pub split: bool,
    /// `IsQuasisplit`: the form is the quasisplit form of its inner class.
    pub quasisplit: bool,
}

/// Compute the presentation of every real form of an inner class, indexed
/// by external form number.
pub fn build_presentations(
    inner_class: &InnerClass,
    classification: &CartanClassification,
    order: &ExternalFormOrder,
    layout: &InnerClassLayout,
    budget: &IntegerLatticeBudget,
) -> Result<Vec<RealFormPresentation>, StructureError> {
    let datum = inner_class.datum();
    let rank = datum.lattice_rank();
    let invariant =
        |reason: &'static str| StructureError::LayoutInvariantViolation { invariant: reason };

    let mut presentations = Vec::with_capacity(order.form_count());
    for external in 0..order.form_count() {
        let internal = order
            .internal(external)
            .ok_or(invariant("external form number"))?;
        let most_split = classification
            .most_split(internal)
            .ok_or(invariant("most split Cartan"))?;
        let cartan = classification
            .cartan_class(most_split)
            .ok_or(invariant("most split Cartan"))?;
        let ms_tau = cartan
            .representative()
            .root_involution()
            .involution()
            .weight_matrix();

        let mut compact = true;
        let mut split = true;
        for (row, ms_row) in ms_tau.iter().enumerate() {
            for (column, &value) in ms_row.iter().enumerate() {
                let diagonal = i32::from(row == column);
                compact &= value == diagonal;
                split &= value == -diagonal;
            }
        }
        if ms_tau.len() != rank {
            return Err(invariant("most split involution rank"));
        }

        let connected = crate::topology::dual_component_group_trivial(ms_tau, datum, budget)?;
        let grading = order
            .special_grading(external)
            .ok_or(invariant("special grading"))?;
        presentations.push(RealFormPresentation {
            name: form_type_name(layout, grading)?,
            compact,
            connected,
            split,
            quasisplit: external == order.quasisplit_external(),
        });
    }
    Ok(presentations)
}

#[cfg(test)]
mod tests {
    use crate::adjoint_fiber::AdjointFiberBudget;
    use crate::cartan_classification::CartanClassificationBudget;
    use crate::{
        BasedRootDatum, CartanClassification, Coweight, InnerClass, LatticeInvolution, Weight,
    };

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn presentations(
        datum: &BasedRootDatum,
        involution: LatticeInvolution,
        roots: usize,
        weyl: usize,
    ) -> Vec<RealFormPresentation> {
        let integer = integer_budget();
        let inner_class = InnerClass::new(datum.clone(), involution, roots).unwrap();
        let budget = CartanClassificationBudget::new(
            integer.clone(),
            AdjointFiberBudget::new(integer.clone(), 50_000, 100_000),
            weyl,
            64,
            64,
        );
        let classification = CartanClassification::build(&inner_class, &budget).unwrap();
        let order = ExternalFormOrder::build(&inner_class, &classification).unwrap();
        let layout = InnerClassLayout::build(&inner_class, &integer).unwrap();
        build_presentations(&inner_class, &classification, &order, &layout, &integer).unwrap()
    }

    #[test]
    fn simply_connected_a1_presents_compact_then_split() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let forms = presentations(&datum, LatticeInvolution::identity(&datum).unwrap(), 2, 2);
        assert_eq!(forms.len(), 2);

        let compact = &forms[0];
        assert_eq!(compact.name, "su(2)");
        assert!(compact.compact);
        assert!(compact.connected);
        assert!(!compact.split);
        assert!(!compact.quasisplit);

        let split = &forms[1];
        assert_eq!(split.name, "sl(2,R)");
        assert!(!split.compact);
        assert!(split.connected);
        assert!(split.split);
        assert!(split.quasisplit);
    }

    #[test]
    fn adjoint_a1_split_form_is_disconnected() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let forms = presentations(&datum, LatticeInvolution::identity(&datum).unwrap(), 2, 2);
        assert_eq!(forms.len(), 2);
        assert_eq!(forms[0].name, "su(2)");
        assert!(forms[0].connected);
        assert_eq!(forms[1].name, "sl(2,R)");
        assert!(forms[1].split);
        assert!(!forms[1].connected);
    }

    #[test]
    fn simply_connected_b2_presents_three_forms() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let forms = presentations(&datum, LatticeInvolution::identity(&datum).unwrap(), 8, 8);
        assert_eq!(forms.len(), 3);
        assert_eq!(forms[0].name, "so(5)");
        assert!(forms[0].compact);
        // The split form Sp(4,R) = so(3,2) comes last.
        assert_eq!(forms[2].name, "so(3,2)");
        assert!(forms[2].split);
        assert!(forms[2].quasisplit);
        assert!(forms[2].connected);
    }
}
