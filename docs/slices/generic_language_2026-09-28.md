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

### Constructor integration and declaration sequencing (in progress)

R4 capture3835245 COMPLETE21s at atlas-generic-sequence-r4-20260928.iOSwHs6V,
pin0f86b6a85a55aa0a41c9b6f2c1fd44d247315451dd1bd3765d28511dab58babc.
All62 original intents now confirmed37accept/25reject;15 prior full-stream
matches retained, using unchanged3835058candidate. Six classifier tests include
the exact indented name-error envelope and false-positive checks; three bridge
checker tests also pass. Report
4a077edf0c9f07c4e847c74f1e7bfb9011e27a8221b358124ce821fa115b0941.
This closes the classifier/capture issue, not constructor runtime acceptance.

Constructor build3835225 FAILED5:09 after successful compilation and43type/
3coercion passes. Syntax47pass1fail: the old manually scoped bare-T unit expects
an error at TYPE_VAR, but TYPE_VAR is now a legal cast prefix (parser.y type),
so missing ':' fails later. Both new parser action/application units pass. No
session/full-core/CLI/capture runs reached. Report
ea23464991656befd97b985f750c72688f76f7a1eb253494cb59bc80067fe385.
R2 job3835267 retains rejection of bare T, adds the binding-position TYPE_VAR rejection
and fresh-command restoration, and preserves Applied required contexts after
structural matching. Its stage is atlas-constructor-build-r2-20260928.hoUj0ZQt,
pin4eaeb6d8ce4be9911eaea0400f35143ff7cb1f547ed83c7b2fe1525a7d1a0f05;
collect its new submission receipt. Do not rerun the terminal first build.

The first build3835225 used atlas-constructor-build-20260928.u6JMxOEe,
pin f5470e5c7383692b0ce9f9bd8f89323acf8fd6e06e58dc1b555639758627758a.
Candidate connects constructor formal reductions to shared lazy scopes, parses
and validates applications, retains arity/names, and uses Cow for structural
consumers: ordinary types borrow; substituted applications own their expansion.
It ports maximal relation-token scanning, retaining single angles as comparison
operators. Six new units, full-core gate and62-case capture requested; no passing
candidate claim. any_type bodies, sequential commands, generic recursive groups
and scheme-carrying inference remain incomplete.

Capture3835154 FAILED3s before execution: local math_suite.py hash disagreed
with the older frozen helper. Report
da79bd1c6a2ca930091627e53413f7107cd5c33f0f93d638ae3e2067d3fc5d55.
Capture3835190 FAILED28s at atlas-generic-sequence-r2-20260928.7OYFmnTc
retains61 three-arm captures using the unchanged3835058candidate. Aggregate
CAPTURE_FAILED_ORACLE_EXECUTION: ordinary name rejection lacks a literal Name
error heading, conservatively classified OTHER_FAILURE. Report
0b43d2526842cbd788aba3997093feeb023d0b266f34bd99d42521f0cf5a34c2.
Capture3835224 FAILED35s at atlas-generic-sequence-r3-20260928.fngY244h
adds the spaced companion (62cases,61classified oracle intents confirmed), but
the new classifier missed two leading spaces. Preserve report
bd58cdcf27c779edde7c186ef1070e6764c02c4a737c66ef46219b02877b8163.
The local checker now retains the exact raw error envelope; needs HPC rerun.
Build3835225's frozen checker predates that indentation fix, so will still flag
that case if it reaches capture. Never change its submitted stage.

Raw original results establish these next implementation contracts:

- Comma-dependent generic definitions give COMMA7seven. With a bad middle
  rigid T:1 initializer, partial_first and partial_last still install and give
  FIRST7seven and LAST11eleven; partial_bad has no overload. T restores to29.
- Ordinary parallel SET installs neither sibling when the second initializer
  calls ordinary_first; both queries report no overloads, recovery prints31.
- `set any_type=3` rejects at ANY_TYPE, but unchanged Rust accepts it. Keep
  that wrong-acceptance fixture; keyword/AST/analyzer repair is not done.
- Row constructors give index/slice/length3,[3,5],3, mutation[7,3,5], and a
  function-constructor call12. Adjacent >>> fails only the nested application
  command. Spaced closers yield NESTED13 and all commands succeed; T restores
  to17. Keep both sources, not just the accepted companion.

Next analyzer/event boundary: current TypedContext::execute returns either a
whole event vector or one error, while Session dispatch appends one diagnostic
on Err. An any_type sequential block must retain earlier reports AND emit an
intermediate diagnostic AND keep executing later bindings. Do not reuse ordinary
execute_set's zipped parallel group or return on its first error. Preserve the
source order per raw binding (not per SET token), including declaration reports,
printed runtime effects and recovery. global.w:926-930/do_global_set is the
executable reference; each initializer gets its own abstraction wrapper.

### Complete-core gate and declaration expectation migration

Capture3835038 COMPLETE16s independently confirms the exact historical unit
source: `p: Pair` prints `Declaring identifier 'p': Pair`, and all four values
are(3,4),3,14,(3,14). Rust's complete stdout/stderr match. All56 oracle intents
agree (35accept/21reject);15 whole-stream matches. Report
86bddb98e26b3c246c56db0d594c19614d8aa22ba838d5c77b26f6315ac5df17.
Only then was the unit's expanded-type report expectation changed; no runtime
behavior or value assertions changed. The failed full-core review is retained.

Build3835058 COMPLETE7:45 at atlas-lexical-core-build-20260928.QM7IzLdg, pin
550899263c93e43a298b1481e353ab936435e19013f6637e0b403e649cf18fa6.
The pinned driver now has an opt-in full-core gate using the separately tested
inventory/summary checker: require every non-known-failure test to execute,
then execute BOTH known polymorphic assignment failures at their exact markers.
Neither may be ignored or called repaired. Current candidate inventories407
tests (six new lexical scope tests):405 pass,0ignored; both known failures
execute at their exact assertions (exit101).150 filtered checks and CLI build/
check also pass. All56 original intents confirmed35accept/21reject;15 full
stdout/stderr matches. The full core command takes13.782s and155048KiB RSS;
all1227 source files and pinned inputs reverified. Report
0dc45ae377d879bc213e893b03ec6096d1619f403582950b8c64939195d6400f.
FPP/F4/E6 retained mathematical units still pass. This is not full generic
support, all-mathematics acceptance or a release speed benchmark. Job terminal;
do not duplicate it. Next work remains actual generic grammar/AST/actions and
the scheme-carrying analyzer, with the source-backed boundaries below.

Next parser integration audit: Cargo.lock pins lalrpop-util0.22.2. Its
state_machine.rs:237-278 fetches and classifies lookahead BEFORE all reductions
on that lookahead, then shifts it without reclassification. Therefore introduce
formals only at reductions whose lookahead is the opening delimiter (or `>`
for constructor type_args). A new generic grammar must pass the same ParserTypes
to its actions and TokenStream, and must not assume Bison default-reduction
timing for virtual groups. The scope tests verify stream timing manually, not
those not-yet-written grammar actions. Also split `<` and `>` into distinct
parser terminals while retaining them in the formula-operator nonterminal;
the current generic Operator token cannot serve both constructor delimiters.
Keep `>=` as one operator and preserve the rejected discovery fixture.

Two further integration boundaries from executable original code:
parsetree.w:2806 wraps EACH declaration initializer in its own abstraction;
global.w:926-930 then executes EACH declaration node sequentially, including
comma-separated bindings, not merely one SET group at a time. Add a comma-
dependency and partial-failure probe before implementing that command; the
current sequential fixture covers separate SET clauses only. Ordinary SET's
parallel binding behavior must remain distinct. For `f@ T (T)`, parser.y drops
the declaration count after lexical setup, and axis.w:7364 wraps the signature
at the surrounding fixed count fc; it is not another type-abstraction body.
Exact overload lookup shifts stored free variables by fc before comparison.
Do not add a synthetic rigid abstraction around this operator-cast form.

### Lazy lexical scopes and additional boundary evidence

Capture3834951 COMPLETE29s at atlas-generic-scopes-20260928.zQdkmNmH reuses
the exact named-type R2 candidate. All55 original arms execute;54 provisional
intents agree, and the proposed bang-on-next-line acceptance is DISPROVED.
Report SHA0f126fe0d88e2507ed32958a836ca3365c1196a4b5f513ef6e0e9b43f7ea32fc.
Keep that source and frozen provisional capture. The newer catalog correctly
records rejection: original says unexpected newline, expecting '!'. Ordinary
multiline constructor fields/formals are accepted, but the mandatory bang must
precede the terminating physical newline once the type spec is complete.

Other new original-backed independent results:

- Multiline pair projectors return7 and"seven"; formal T can then bind11.
- Sibling `any_type T` scopes are both legal and produce (2,"abc"); outside
  them T can bind19. This is distinct from the rejected NESTED same-name scope.
- An any_type block installs its declarations sequentially: scope_pair calls
  preceding scope_identity, producing (2,"abc") and(true,3/4). T/S can then
  bind13/17 outside the block.
- A generic recursive group expands the implicit MathGenericList reference
  into MathGenericList<A>; its int instance extracts5, with T restored after.
- A TYPE_VAR used as a value binding is rejected at TYPE_VAR, not at the
  preceding any_type declaration as Rust currently does.

Lexical-scope build3834975 COMPLETE6:50 at
atlas-lexical-scope-build-20260928.BKCNlPyQ, pin
2c2184a056b01dddb3e928dd59ef441f88c81ddc39e5aa4d95b68949351da74d.
The candidate moves persistent name classification from an eager whole-command
rewrite into TokenStream::next and adds shared ParserTypes with scanner-time
groups, parser-time formal installation, duplicate slots/outer offsets,
constructor/variable distinctions and reset. Six new unit tests include actual
lazy-token consumption and TYPE_VAR expression rejection. Existing generic
declaration actions/AST/inference are NOT connected yet; no generic support
claim. The raw Lexer still collects command boundaries separately. Next grammar
integration must verify LALRPOP action/lookahead timing against the captured
multiline and next-line-bang cases, not infer it from Bison's default reductions.
All149 related tests and CLI check/build pass. All55 corrected oracle intents
match (34accept/21reject),14 full streams remain equal. Report
b6bc69edcb2f02ad4b2022390b9e7fa55eb43f456b76efb1ec3fc862d6c0c16e.
Source/patch/current55case inputs are pinned; job terminal, do not duplicate.

### Named lifetime and structural consumers: current migration

Before3834815 COMPLETE22s reuses the exact verified3834754debug binary and
captures45cases (26original accepts/19rejections). All original intents match;
the nine added inputs retain full streams/time/RSS. Simple named union
discrimination and type-name reuse after forget are Rust failures, not merely
different report text. Ordinary row/function/field/history values agree, but
names, definition locations and identifier queries differ. Report SHA
23008c2c5d3173032330d63499fd432aeb6a5c641ff1f1b00d9d6250616a13ff.

Candidate3834852 FAILED2:47:41type/3coercion/40syntax tests pass, session35pass/
3fail. Two assertions reveal that definition spans lack the consumed newline;
one new test mistakenly expects Output instead of the session's ReportLine.
The driver stops before CLI build/capture. Preserve report
ba6693137283cf23b172186ab5e2edea13d01f3a28689732d16a4839aa032ecc.
Parser.y's set_type actions use @$ including the newline. Forward the actual
lexer terminator through normal/session-frame command execution; do not guess
last-token-end+1, which loses trailing comments and whitespace.

Edge capture3834868 COMPLETE21s confirms all49 original intents (30accept,
19reject), report14cf1eeb550d5fe4cdfc739665316a30438743db9ee0b0d8c211214cb2330b1f.
Two provisional explanations were false, and the unchanged sources remain:

- Named-void global bindings retain42, [1,2] and the function value. Original
  axis-types.w::coerce accepts void without wrapping an expression; handling
  void at evaluation boundaries is separate. Do not impose structural-void
  discarding on these named global initializers.
- Redefining MathMemberRecord leaves member_second overloaded at its old
  MathMemberRecord argument and preserves user-replaced member_first. Actual
  global.w::clean_out_type_identifier first returns when kind()!=tabled;
  kind() exposes the structural top, so the documented projector cleanup is
  bypassed for this example. Port the observable behavior, not that prose.

R2 candidate3834895 COMPLETE6:30 at atlas-named-build-r2-20260928.xa3ImNjp,
pin a4ee2a14a2bab18d47e776dd38b0148aee6adc86e33e71a5035d785513af195a.
It retains type identities plus active name bindings, copies fields, exposes
structure only where needed, preserves names in successful casts/reports, uses
semantic (not slot/textual) equality for overload matching and implements the
captured edge behavior. All143 related tests (42type/3coercion/40syntax/
39session/19session-frame) and CLI check/build pass. Report SHA
386976d612e9438e0c1acd4d91c4331ac4f8e69440a6380013f686c86714c8ab.
All49 original intents confirmed;14 complete stdout/stderr matches versus2
before. These include named annotations, environment/recursive-result controls,
structural equality, rows, function values, copied fields, union discrimination,
forget/reuse, overload replacement, retained members and primitive contexts.
The union and forget failures are fixed on unchanged retained inputs.

Six named rejected inputs now reach the right rejection category and matching
stdout, but stderr prose differs. Named history retains correct old/new values
but ordinary identifier whattype metadata still differs. Named void retains
scalar/row/function values correctly, but general closure printing lacks the
original location/body. Grouped-type canonicalization and atomic field-conflict
handling also need broader coverage; do not declare all named types finished.
Candidate timing is DEBUG versus release oracle/old Rust; no speed ratios.
Full-core review3834919 FAILED1:00 using the same R2 binary/source hashes.
3 checker tests pass;401 Rust tests inventoried;398pass/1fail/2filtered in
16.83s with154644KiB RSS. The sole failure is a historical unit asserting
`Declaring identifier 'p': (int,int)` while the runtime retains Pair. Its full
six-command source is added as named_declaration_fields (catalog56) for exact
original confirmation before updating that assertion. Separate known-global/
local expected-failure invocations were not reached; this review does not prove
they executed. Report SHA
6455ab36eb8d096f1a850fcae14146d459c79c168a3a59ea8744c3d77ac79c6b.
The same pending job was moved from fat (projectedOctober1 start) to cpu8GiB;
actual allocation remained4CPUs because NumCPUs does not lower CPUs/Task.
No duplicate submission or source/binary change. Full generic inference and
high-level mathematics remain unproven.

### Persistent-environment integration: scoped progress, remaining output gaps

Job3834754 COMPLETE6:17, pin
c23be725d1153c89df51bf5b7d2015eec13c074ccea2819f2628449d59db1d97.
`parse_command_in`/`parse_expression_in` classify known type names separately
from IDENT, at the session/redirect boundary. Only grammar positions matching
original `id` accept either class; expression/parameter bindings remain IDENT.
Casts, parameters and recursive results now call the live-table resolver and
cannot silently turn an unknown name into Undetermined. Five tests added
before implementation, plus five source probes (catalog36). HPC passes38type/
3coercion/40syntax/36session/19session-frame tests and CLI check/build. Its
capture is invalid: all36 oracle arms fail at loader startup with missing
GLIBCXX_3.4.26/29. The build-only batch environment lacked the GCC runtime
library path. Report SHA
24929729a67f2ce0d28fcec03d0008903088b4b0ecb6499e3e3742e1600c964e
is retained as environment failure, not new oracle contracts. Replay3834785
COMPLETE52s in a fresh stage uses the same source/binary, corrected library path,
rehashed build evidence and9 checker tests. All36 oracle intents confirmed
(19accept/17reject). Report SHA
bf738be8f6ab362be0cfe0b2988f2881b0611daec378b4d9a47be986f2a2db1e,
`tests/reference/hpc/math_language_bridge_replay_2026_09_28.json`.

The new candidate executes named row casts/parameters and recursive results,
printing NAMED_TYPE[2,3][4,5], NAMED_ENV["x"] and NAMED_REC[1]. Wrong named
components now produce type errors, not syntax errors. The old Rust wrongly
accepted TYPE_ID as a parameter binding; the unchanged fixture now rejects it.
The full streams still differ: names expand away in variable/function reports,
queries use the old format, redefinition says defined instead of redefined,
and diagnostic formatting/locations differ. A bare TYPE_ID's missing colon is
reported at the file's EOF rather than the original's command newline; keep
this explicit. Original stderr for the two TYPE_ID syntax cases contains about
25KiB of leading caret indentation; raw bytes/hashes are retained, not normalized
as acceptance. Only the two pre-existing concrete mutable controls match whole
streams. Both jobs are terminal. No full generic/math/performance acceptance.

This first integration does NOT implement intra-command TYPE_VAR scopes or
scheme-carrying expression inference. The legacy alias map still expands
names, so named reports remain incompatible. Original global.w:1485-1518
and axis-types.w:1507-1618 retain even zero-arity definitions as table entries:
add_simple_typedef expands the top constructor only, preserves nested names,
and deduplicates using name plus textual structure, not structural equivalence
alone. Distinct nonrecursive named types can nevertheless be structurally
equal; distinct recursive definitions remain nominal. Name lookup/redefinition
must not mutate the meaning of types already stored in variables. Port those
semantics and the query/projector/forget paths together; switching add_alias to
the current legacy Tabled path alone is unsound (its comparison is nominal and
several consumers inspect unexpanded raw variants).

Query formatting has also changed: latest global.w:2106 `type_of_type_name`
prints the definition location, retained name/formals and expansion (fields
vertically), not the legacy Rust `Defined type:` line. The bridge's unit gate
only checks its two query syntaxes reach the same legacy query path; the full
R8 capture must retain this output mismatch. Do not take that unit expectation
as the latest-original golden when migrating type-definition metadata.
Similarly, global.w comments about removing a type-map entry must be read with
the implementation: Id_table::remove erases the active identifier binding;
old table entries remain available to values/types already holding them.

Next scoped-token migration must be lazy, not a whole-command name rewrite.
Original lexer.w:499-520 pushes clutches for LET, BEGIN/IF/WHILE/FOR/CASE,
pops LET at IN and block clutches at END/FI/OD/ESAC; parentheses/brackets do
the same at scanner time (672-677). parser.y:933-950 installs type variables
only after the complete declaration list and its opening lookahead have been
seen. lexer.w:530-614 assigns indices by outer-clutch sizes and first occurrence
within the current clutch; duplicate slots remain counted, unused, and scoped.
A parser-local shared environment plus a lazily classifying TokenStream can
carry those syntax-only effects without mutating TypedContext. However the
outer raw Lexer currently collects the entire command first: constructor
formals' virtual nest also affects newline termination and recovery. Audit
that command boundary while adding ANY_TYPE/TYPE_VAR/TYPE_CONSTR; a token-only
second pass must not silently diverge on multiline constructor declarations.

For active inference, follow executable CWEB sections rather than stale prose:
axis.w:4097 claims the surrounding type is not passed into a type abstraction,
but the actual case at4110 raises the existing tp floor, calls convert_expr on
that same tp, then lowers its assignment floor. The Rust port must preserve
that context and pending-substitution behavior (the internal raise/lower API is
already tested), not replace it by an unrelated unconstrained inference pass.
Likewise the current first-exact/first-coercible overload scan must be replaced
by the new scheme-aware ambiguity rules; R2 proves a concrete int overload does
not automatically win against a generic one. Preserve that negative contract.

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

Pre-bridge parser audit: `session::execute_tokens` called
`parse_command(tokens, source)` without the live type table; `TypeExprNode`
excluded defined names and only `SpecTypeNode` allowed a raw identifier.
The bridge above fixes the persistent environment. A keyword-only `any_type`
patch still cannot resolve scoped generic
constructor casts or type variables in lambda signatures. Preserve the LR
type/expression distinction via a scoped type-name/type-variable environment,
including scope changes inside one command; do not blindly add identifiers
to every type production. Original TYPE_VAR versus IDENT classification is
observable in the same-name nested-abstraction rejection. The session's
command-at-a-time boundary is necessary but not sufficient for that behavior.
The bridge replaces semantic calls to `TypeExpr::resolve()` in casts, lambda
parameters and recursive-function result analysis: that diagnostic-only helper
uses an empty table and silently falls back to Undetermined. These paths now
resolve against the actual environment and report unknown names. Both normal
session commands and session-frame redirection parsing are connected. Keep the existing
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
