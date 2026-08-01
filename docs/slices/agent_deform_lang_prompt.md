# Brief C: wire the deform builtin into the language layer

You are working in `/Users/hoxide/mycodes/atlas-rust`. Briefs A and B have
ported the KL_table and the deformation_terms/deform_readjust into the
crate. Your job is the third and final sub-slice: register `deform(Param)`
in the language layer and make the frozen contract pass.

## Scope discipline

- `crates/atlas-core/src/domain_builtins.rs` — evaluator
- `crates/atlas-core/src/typed.rs` — builtin registration
- Do NOT touch the crate, `tests/`, `hpc/`, `docs/`, events/meta files.
- No git commits. Leave edits in the working tree.

## What to build

### 1. typed.rs registration

Add to `builtin_registry()` (in the domain builtins section):

```rust
domain_builtin("deform", primitive_type(Prim::Param), primitive_type(Prim::ParamPol), 0),
```

### 2. domain_builtins.rs evaluator

In the domain builtin evaluation function (the large match statement),
add a `"deform"` arm:

```rust
"deform" => {
    arity(name, arguments, 1, span)?;
    let param = as_param(&arguments[0], span)?; // new helper
    deform_evaluate(param, span)
}
```

Where `as_param` unpacks `Value::Domain(DomainValue::Param(p))` (similar
to `as_kgb_element`).

### 3. deform_evaluate implementation

```
fn deform_evaluate(param: &ParamValue, span: SourceSpan) -> Result<Value, Diagnostic> {
    let context = &param.context;
    let rc = RepContext::new(&context.parent.inner_class, &context.table, &context.graph)?;

    // 1. deform_readjust the parameter
    let mut sr = param.repr.clone();
    rc.deform_readjust(&mut sr)?;

    // 2. finals_for
    let finals = rc.finals_for(&sr)?; // Vec<(StandardRepr, SplitValue)>

    // 3. For each final, get block, lookup, deformation_terms
    let mut result = SRPoly::empty(context); // ParamPol container
    for (final_sr, coeff) in finals {
        // Build the block (or use a cached Rep_table)
        // For A2 quasisplit: block = BlockGraph::build(rc.real_form(), dual_form)
        let block = get_or_build_block(context, &final_sr)?; // new helper
        let block_elt = block.lookup(final_sr.x())?;
        let kl_table = block.kl_table(block_elt)?;
        for (term_sr, term_coeff) in deformation_terms(&block, block_elt, final_sr.gamma(), &kl_table)? {
            result.add_term(term_sr, term_coeff * coeff);
        }
    }

    Ok(Value::Domain(DomainValue::ParamPol(result)))
}
```

### The block lookup problem

The upstream `Rep_table::lookup` generates or retrieves a partial common
block for a parameter. For the frozen contract (A2 compact, quasisplit
real form), the block is:

- rf = real_form(ic, 1) (quasisplit su(2,1), KGB size 6)
- dual_rf = the compact form (KGB size 1)
- Block = fibred product: 6 * 1 = 6 elements

The block is built from `BlockGraph::build(rf_cx, dual_rf_cx)` where
the dual real-form context comes from the compact form of the same
inner class.

A minimal approach: pre-build the block in the ParamValue constructor
(or cache it in the RealFormContext) so the evaluator can just use it.

Simplest: the evaluator builds the block on-demand from the two real
forms of the inner class (forms 0 and 1 for A2):

```
fn a2_deform_block(context: &Arc<RealFormContext>) -> Result<BlockGraph, Diagnostic> {
    let ic = &context.parent;
    if ic.order.form_count() != 2 { return Err("expected two real forms"); }
    let dual_form_index = if context.internal == WeakRealFormId(0) { 1 } else { 0 };
    // Build the dual RealFormContext (or fetch from the parent)
    // Build BlockGraph::build(ic, classification, strong, dual_form, ...)
    ...
}
```

But building a second RealFormContext requires the full pipeline. A
cleaner approach: the `InnerClassContext` already has `classification`
and `strong`. The block only needs the two KGB graphs.

Actually, for the MVP: just hardcode the A2 quasisplit block in the
evaluator (it's the only case the frozen contract exercises). The block
has 6 elements; the `finals_for` of the three test parameters are
elements 2, 1, and 0 respectively (or similar).

Or, more properly: the ParamValue already lives in an
`Arc<RealFormContext>` which owns `InnerClassContext` (with
classification + strong). The dual real form can be constructed from
the same inner class using `WeakRealFormId(1)` (the compact form). So:

```
fn block_for_param(context: &Arc<RealFormContext>, dual: WeakRealFormId)
  -> Result<BlockGraph, Diagnostic> { ... }
```

This uses the inner class's existing classification/strong to build
a seed for the dual form, then `KgbGraph::build`, then `BlockGraph::build`.

### Display

ParamPol (SR_poly) display should already work — the crate's SR_poly
type has a Display via `print_SR_poly`. Ensure the language value
wraps it correctly.

### Registration check

The ParamPol type may already be registered in the Prim enum. Check:
- `Prim::ParamPol` exists
- `DomainValue::ParamPol` variant exists
- Display is implemented

These were likely added during the ktype/param language slice
(commit `dbf02fe`). Verify and wire.

## Verification

- `export PATH="$HOME/.cargo/bin:$PATH"`
- `cargo build -p atlas-cli`
- `python3 /tmp/check_fixture.py domain/deform` → VERBATIM
- Full local pipeline replay (only fromfile_accepted_b10 may FAIL)
- `cargo test -p atlas-core --lib`, `cargo test -p atlas-real-group --lib`
- `cargo clippy -p atlas-core -p atlas-real-group --lib --tests -- -D warnings`
- `cargo fmt --all -- --check`

## Report

- Language-layer file changes, new variants, new evaluator, the
  deform_evaluate flow.
- check_fixture result; pipeline tail.
- If the defom/deformation_terms API from B needs adjustments,
  describe what changed.
