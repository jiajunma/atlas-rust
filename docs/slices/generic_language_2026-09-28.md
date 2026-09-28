# Latest-original polymorphism: contracts and implementation boundary

This is a prerequisite for unmodified latest `basic.at`, hence unitarity,
Hodge, AV-ann and associated-cycle scripts. Bare-core KL repairs do not close
this language gate. Original revision7e1b958c, Rust capture binary from
build3833612; the mathematical runtime repairs committed in3ec91f8b do not
change its language engine. All captures and unit probes run on HPC.

## Observed contracts

Capture3833740 discovered11 cases;3833758 retained those sources and added5.
The latter ran6 checker tests, rehashed606 original/1212 Rust source files,
and pinned binaries and264 upstream scripts. Its report SHA is
c321581d1666d9856951701ba1ab0c83955643f5447ba20b9b77f381de3d29d6.
Raw streams, exact GNU-time RSS and wall seconds remain on HPC; metadata is
`tests/reference/hpc/math_generic_probe_r2_2026_09_28.json`.

| Contract | Current original | Current Rust |
|---|---|---|
| Nested `Pair<S,T>` with projectors | Accepts nested row/tuple arguments | Syntax/name failure |
| Identity at independent int/rat/row instances | Accepts; same generic definition reused | Syntax/name failure |
| Return-only variable constrained by result context | Produces `[int]` and `[rat]` empty rows | Syntax/name failure |
| Generic plus concrete int overload, ordinary int call | Ambiguous argument, analysis rejection | Fails earlier; not equivalent |
| Explicit `#@ T ([[T]],[T])` | Accepts int and string row instances | Syntax/name failure |
| Rigid `T:1` in a generic function body | Type error, no definition installed | Syntax failure |
| Repeated `T` instantiated with int and string | Type error | Fails earlier |
| Too few/too many constructor arguments | Exact arity analysis errors | Syntax failure |
| Type formal used after its scope | Syntax rejection AFTER a valid declaration | Also fails the declaration; not equivalent |
| Duplicate formal names `<T,T>` | Accepts, prints `(A,A)` | Syntax failure |
| Independent `S,T` arguments | Accepts `(2,"abc")` and `(true,3/4)` | Syntax/name failure |
| Repeated `<T,T>` instantiated with `<int,rat>` | Both projector result types are `int` | Syntax/name failure |
| Explicit generic selection `probe_select@ T (T)` | Selects generic; int instance returns2, not3 | Syntax/name failure |
| Identity used where `(S->[S])` is required | Type rejection; no inferred recursive type | Fails earlier |
| Assignment to `set probe_empty=[]` | Constant-binding analysis rejection | WRONG ACCEPTANCE |

The historical ID `duplicate_formals_rejected` deliberately remains: its
initial rejection hypothesis was disproved, not silently deleted. Likewise,
`concrete_overload` now has rejection intent based on the actual oracle.
Constructor-arity diagnostics have no literal `Type error` heading; the R2
classifier recognizes their exact structure separately. Classification uses
stderr, not user-printed error words. A label match is never enough to prove
equivalence of rejected programs or their preceding successful declarations.

R3 job3833787 COMPLETE adds `monomorphic_empty_assignment.atlas`, the required control:
`set probe_ints=[int]:[]; probe_ints:=[1]` must remain accepted. Its receipt
records the17-case capture; this control's complete streams match. Report SHA
9184eb016e262418882e8b2cf385512d3da16bafc20c1d71edeccf688785ef49.
Unit probe3833788 COMPLETE2:18 adds ONLY
`typed::tests::polymorphic_empty_global_is_not_assignable` to mathematical
candidate3833716. It first permits concrete assignment, then executes the
polymorphic rejection assertion failure: `constant=false rejected=false`,
0passed/1failed. Report SHA
1ab2f23b5b520f0a29a88ef6469ab41bddbbce5012b6e0043a650b78d7c301c0.
No repair/after-pass yet. All these jobs are terminal; do not duplicate.

## Source-backed causes and port order

1. **Type representation and assignment scope.** `axis-types.w:2264-3010`
   distinguishes independent undetermined holes from repeated type variables.
   Variables below the context threshold are rigid; those above it can be
   instantiated. Substitution follows assigned variables transitively and
   rejects direct/indirect occurrence cycles. Shifting fresh variables must
   leave fixed variables unchanged. Rust `types.rs::Type` currently has only
   `Undetermined` holes and monomorphic structural/tabled types, so replacing
   every occurrence of `T` with a hole would lose the repeated-variable rule.
   Introduce owned type schemes and per-analysis substitution state. Clone or
   roll back state for candidate overload trials; do not mutate a global scheme
   when instantiating one use. Rust-owned enums, vectors and explicit scope
   objects suffice; no unsafe code or global mutable type-substitution cache.

2. **Constructors keep declared arity.** `type::constructor` preserves the
   supplied degree even if some parameters never occur in its body. Tabled
   constructor applications carry arguments; recursive identity/expansion must
   avoid infinite unification. Rust `TypeTable` has no constructor arity or
   application arguments. The repeated-name probe proves that unused slots
   cannot simply be dropped or arity inferred from distinct names. Preserve
   named types, field/projector types and nested substitution, not just tuples.

3. **Scoped syntax, not script rewriting.** `parser.y:171-176,435-456,887-945`
   and `parsetree.w:3369` define command/expression abstractions, type arguments
   and explicit polymorphic operator selection. `lexer.w::put_type_variable`
   binds names in a temporary lexical group. Rust `parse_command` has no type
   environment; `TypeExpr` lacks applied constructors and scoped variables.
   Add explicit AST/scope handling and preserve original command grouping and
   recovery. The old lexer unit claiming `any_type` is an identifier is tied
   to the historical oracle and must migrate with current positive/negative
   tests. Do not strip or preprocess generics out of upstream `basic.at`.

4. **Analysis and overloads.** `typed.rs::Analysis`, `convert_expr`,
   `convert_lambda_expression`, `convert_overload_application` and global
   bindings must carry/instantiate schemes consistently. The current resolver
   chooses the first exact ordinary overload;3833758 proves that this rule
   cannot be extended unchanged to polymorphic/concrete overlaps. Both argument
   and result contexts matter. Casts currently call `target.resolve()` without
   the analysis type table; applied/scoped types must use the proper environment.

5. **Polymorphic bindings are implicitly constant.** `global.w:984-1005`
   wraps each binding leaf at fixed threshold0, then sets the constant flag
   when the resulting scheme is polymorphic. Current `execute_set` passes only
   the pattern's explicit constant flag to `define_variable`; the latter never
   recognizes polymorphism. The empty-list error is thus a binding/type-model
   defect, not a list mutation implementation defect. Explicit concrete casts
   remove polymorphism and retain mutability. Inside an abstraction a fixed
   type variable is not itself polymorphic; do not mark all variable-containing
   types constant. Preserve rejection text and binding/report types as well as
   the runtime prohibition. A special case for the value `[]` is insufficient.

## Acceptance still required

Retain all discovered failures and the before unit unchanged. Add nested
scope/shadowing, local bindings, higher-order inference, fresh instantiation,
unused constructor parameters, recursive constructor and failure-recovery
cases as each boundary is implemented. Re-run complete streams against the
pinned original, including cases72-74 (`basic.at` loading). No generic feature
is supported until those relevant positive AND negative comparisons pass.
Then resume the existing high-level mathematical inputs without simplifying
scripts, groups, parameters, coefficients or cutoff checks.
