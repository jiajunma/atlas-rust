use atlas_real_group::{InvolutionId, InvolutionTable, StructureError};

#[test]
fn compact_elected_word_is_available_at_the_language_boundary() {
    let _: fn(&InvolutionTable, InvolutionId) -> Result<Vec<usize>, StructureError> =
        InvolutionTable::weyl_word;
}
