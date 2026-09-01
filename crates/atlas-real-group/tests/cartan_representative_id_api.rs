use atlas_real_group::{CartanId, InvolutionId, InvolutionTable};

#[test]
fn canonical_cartan_representative_id_is_public() {
    let _: fn(&InvolutionTable, CartanId) -> Option<InvolutionId> =
        InvolutionTable::cartan_representative_id;
}
