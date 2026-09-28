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

### Internal foundation implemented, language integration still open

HPC job3833858 COMPLETE4:12 checks the new `types::polymorphic` module and
`Type::{Variable,Applied}` representation. All17 type units (13new/4legacy),
3 coercion units and `cargo check -p atlas-cli` pass. The unchanged global
wrong-acceptance regression still executes/fails as required; this is NOT an
after-pass. Report SHA
4179f3ec973864a988da06f953589add340f31dd6b340cebc68d62e4a9c1118e.
The report binds the parent archive/probe, exact1213-file source manifest,
patch, driver and submitted script, isolated target and command time/RSS.

`TypeScheme` owns a clean packed body/fixed threshold/degree;
`TypeAssignment` owns per-trial substitutions, fresh instantiation, floor
lowering, transitive occurs checking and substitution/compaction. Rollback
uses an explicit owned snapshot. Constructor applications retain unused
arguments, validate IDs/arity before unification shortcuts and perform
simultaneous substitution. Recursive nominal identity prevents infinite
expansion. The existing analyzer does not yet produce/use these new types.

Remaining integration must carry types AND their assignment scopes together,
following original `type` in axis-types.w:2990-3625. In particular: raising
a fixed-variable floor must first bake pending assignments; combining tuple
components or overload candidates must freshen their free-variable ranges;
extracting a function result must apply the same argument substitutions.
Do not feed compacted substitution output back into the old assignment.
`convert_expr` currently takes a bare `&mut Type`, `Analysis` lacks the
fixed-variable count, and local/global `TypeCell`s hold bare types. Those
interfaces, lambda/let binding, balancing, casts and overload trials must
migrate together. The older `specialise` API is not linked-variable inference.
Primitive `Undetermined` wildcard overload registrations also need explicit
schemes; new type variables cannot be treated as ordinary monomorphic holes.
Recursive-group declarations need another lowering step: original
`simple_subst(...,group)` in axis-types.w:1186-1265 forwards all actual
arguments to implicit same-group references. The new primitive substitution
currently handles explicit `Applied` arguments only. Normalize same-group
references to explicit formal applications when lowering declarations, and
validate forbidden recursion through existing recursive constructors. The
nominal-termination unit is not proof of recursive-declaration support.

Current display implements the original A/B/... spelling including ASCII
punctuation after Z. Non-ASCII/large-index behavior needs a separate contract;
do not claim full diagnostic compatibility from the foundation units.

### Scope-carrying foundation verified (not language acceptance)

The next candidate adds `InferredType { body, assignment }`, porting owned
scope operations from axis-types.w:2990-3435/3625-3641. It includes baking and
repacking pending substitutions before raising the fixed-variable threshold,
lowering the threshold on abstraction exit, disjoint tuple component imports,
and importing BOTH an expression and its pending assignment values. Overload
matching returns the shift needed to substitute the function result; failed
trials can restore their original assignment degree. One-sided unification
has an explicit full-snapshot rollback variant. Clean constructor scopes keep
their unused declared parameter slots, as in the captured phantom contract.

Nine new internal tests were added before the implementation. Compute job3834389
COMPLETE4:15 at `/public/home/majj/atlas-type-scope-20260928.VbWRGzcY`, pinned
by29c461ae28757dfe1b52fba2e953baeba432b8c326ca5dd5127e452bdcaf002d.
The source is the prior global-probe baseline plus the two type foundation
files; it does not change the active analyzer or include the separately retained
local-probe unit. New polymorphic.rs SHAa45e0edf329baaf3ef0fbd6d5e59a0cf077d36604033fdb18a5036cc24864b02.
All27type/3coercion tests and CLI check pass. The known global regression still
executes/fails at its intended assertion; it is not repaired. Report SHA
40946d39c30790af6504e76a3a2130ac7cfc629b2ba263866e8044eff8ccacbe,
`tests/reference/hpc/math_type_scope_2026_09_28.json`. All source hashes match
the tested archive+patch tree; do not duplicate this terminal job.

The matching foundation adds two-sided transactional unification, structural
pattern specialisation (including recursive nominal identity), read-only
formal matching, named/assigned function component access and direct function
argument/result substitution. Eleven new internal tests cover repeated versus
independent variables, pending/rigid scopes, structural exposure and rollback.
Job3834447 COMPLETE4:44 at atlas-type-matching-20260928.sP9ffXRv; pin
47bf30c87a11656837cc88542a40efd7c3ea6934f89f8f5baf8b4d1260e8e5f9,
polymorphic.rs e8d63d0eed149db31c60306a669644148e1c3ec123e5d7fed198739911b18cd0.
All38 type tests,3 coercion tests and CLI check pass; the known global
assignment regression still executes/fails. Report SHA
799066fe0f91e45622f9bbc9b48c18e6ce9c86e47c8995641dd4dde81637b2ec,
`tests/reference/hpc/math_type_matching_2026_09_28.json`.
The safe Rust API uses owned snapshots for
both bodies and assignments on failure, and accounts for differing floors
when importing an earlier inferred argument. This does not change the active
analyzer or establish any new source-language acceptance.

Next migrate Analysis/TypeCell/convert_expr and parser type environments
together. No global/local assignment after-pass or latest basic.at loading
is claimed. The language probe catalog now has31 provisional contracts:
six new inputs cover named monotype annotations and rejection, named function
application, direct local polymorphic calls, independent tuple functions, and
a rigid outer result rejection. Capture3834702 COMPLETE34s confirms all31
oracle intents (17accept/14reject),6 checker tests,606original/1212Rust source
files and264 scripts rehashed. Report SHA
31901e9563db8128d51a4c61d60382984f000acbc7dbea3d8d37e42df3ddd271,
`tests/reference/hpc/math_generic_probe_r7_2026_09_28.json`.
The six new complete original streams were inspected: direct calls return
([2],[3/4]), tuple functions return(2,["abc"]), named function returns[7],
and the int result cannot satisfy rigid A. Named monotypes are retained in
variable/function reports, not just expanded away: latest global.w:1503 uses
add_simple_typedef even for an unparameterized definition. The current Rust
alias map still expands those names, another integration mismatch to fix.
Preserve old captures with their25-case catalogs; use the new31-case checker
only with the matching new catalog. Both matching/capture jobs are terminal.

Parser integration audit: `session::execute_tokens` currently calls
`parse_command(tokens, source)` without the live type table. The general
`TypeExprNode` deliberately excludes defined names; only `SpecTypeNode`
allows a raw identifier. A keyword-only `any_type` patch cannot resolve named
constructor casts or type variables in lambda signatures. Preserve the LR
type/expression distinction via a scoped type-name/type-variable environment,
including scope changes inside one command; do not blindly add identifiers
to every type production. Original TYPE_VAR versus IDENT classification is
observable in the same-name nested-abstraction rejection. The session's
command-at-a-time boundary is necessary but not sufficient for that behavior.
Also replace semantic calls to `TypeExpr::resolve()` in casts, lambda
parameters and recursive-function result analysis: that diagnostic-only helper
uses an empty table and silently falls back to Undetermined. These paths must
resolve against the actual environment and report unknown names. Update both
normal session commands and session-frame redirection parsing. Keep the existing
`whattype TypeName` and type redefinition paths working: upstream TYPE_ID is
allowed by `id`, but not by every IDENT-only binding production. Merely
reclassifying every identifier without adapting those productions regresses
the existing settype_b5 fixture.

### Additional original-backed discovery

R4 job3834266 COMPLETE9s captures22cases;6 checker tests pass. Report SHA
dc809b5c37c51739683dcf94e1d5a6bfdb4783b7ff5898906ae5932f34b6b427.
It accepts fixed-T local mutation and nested DISTINCT-T/S abstractions:
`probe_outer` has `(A->(B->A,B))` and returns `(2,"abc")`/`(true,3/4)`.
Rust still fails their syntax. Original rejects nested reuse of the same
type-variable name (TYPE_VAR where IDENT required); the initial acceptance
hypothesis is disproved, not a request to broaden the original language.
Original rejects `let xs=[] in xs:=[1]; xs` as a constant assignment, while
Rust wrongly returns `[1]`. The new unit retains that failure with an explicit
concrete local assignment control. Root cause also includes
axis.w:3084-3119 `thread_bindings`, not only global.w:992.

R5 job3834282 COMPLETE10s captures24cases;6 checker tests pass. Report SHA
da21cb68d53b0cb378c28924a450f747a34cfc396f99d73403f031801bb22f0b.
The concrete local `[int]` control succeeds with identical full streams.
Two preserved discovery inputs do NOT test phantom-parameter semantics:
the unspaced `T>=` is an operator token; adding whitespace then exposes the
invalid SINGLE named field (struct_specs requires two). A separate two-field
companion is required. These are fixture errors, not original math defects.

R6 job3834288 COMPLETE10s confirms all25 current oracle intents (13accept,
12reject);6 checker tests pass. The two-field companion defines `(A,A)`,
preserves `MathPhantom<int,rat>` in the variable report, returns7/9, and both
projectors have type int. Report SHA
9175f1b4fbdad9ed114580398b093580ee8ea0df1f4c7d8940d6c94920d457a3.
Local unchanged-runtime probe3834285 COMPLETE3:32 executes the new unit:
the concrete control passes, then `rejected=false` and0passed/1failed at
the intended assertion. Report SHA
86d014395dcd80a10fbccae10e99383df336b15a0658251f6e2c817c34c606c9.
No global/local constness repair or after-pass. All capture/local-probe jobs
above are terminal; do not duplicate them.

Foundation R2 job3834296 COMPLETE6:25: a new source-backed
regression checks that Applied specialisation exposes the structural receiver,
not just a true compatibility result. The before phase restores only3833858's
types.rs; the same test executes/fails before and passes after, in isolated
targets. All18type/3coercion units and CLI check pass. Report SHA
adbc4b8f88ea06825ec2623eccf811aca4c7920a972a7ec7dd7f43c748906902.
This is internal foundation verification, not generic-language acceptance.
Legacy Tabled specialisation has a similar read-only path; investigate with a
separate source-language fixture before altering that already-active behavior.

### Source port boundaries (original discovery plan)

1. **Type representation and assignment scope.** `axis-types.w:2264-3010`
   distinguishes independent undetermined holes from repeated type variables.
   Variables below the context threshold are rigid; those above it can be
   instantiated. Substitution follows assigned variables transitively and
   rejects direct/indirect occurrence cycles. Shifting fresh variables must
   leave fixed variables unchanged. The active Rust analyzer still uses
   `Undetermined` holes and monomorphic structural/tabled types (the new
   representation is not integrated yet), so replacing
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
