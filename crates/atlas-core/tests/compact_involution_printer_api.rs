use atlas_real_group::{InvolutionId, InvolutionTable, StructureError};

#[test]
fn compact_canonical_expression_is_available_at_the_language_boundary() {
    let _: fn(&InvolutionTable, InvolutionId) -> Result<Vec<i32>, StructureError> =
        InvolutionTable::weyl_canonical_involution_expr;
}
