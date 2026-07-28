# Axis core oracle trace (upstream master 4d3e9449)

Raw trace notes for language phase B: the type analyser
(convert_expr, coercion table, balancing, overload resolution,
closures, value model, printing) and the builtin inventory.

---

Oracle trace notes: the axis type analyser and runtime (all cites `sources/interpreter/` in `/Users/hoxide/mycodes/atlasofliegroups`, master 4d3e9449; line numbers are `.w` source lines).

# 1. Type representation (`axis-types.w`)

**`type_expr`** = tag + union (axis-types.w:362-388):
- `enum type_tag { undetermined_type, primitive_type, function_type, row_type, tuple_type, union_type, tabled }` (289-293).
- Union fields: `prim_variant` (primitive_tag), `func_variant` (`func_type*`), `row_variant` (`type_p`), `tuple_variant` (`raw_type_list`, used for BOTH tuple and union), `type_number` (tabled) (364-370).
- `func_type { type_expr arg_type, result_type; }` (491-500) — multi-arg functions are just tuple `arg_type`.
- Accessors auto-expand `tabled`: `kind()/prim()/func()/tuple()` go through `untabled()`; `raw_kind()` does not (375-385). Trees are strictly owned, no sharing; deep `copy()` instead of copy ctor (153-158, 400-419).

**Primitives** (295-298 + 3561-3567; names 313-315, 3573-3577): `int,rat,string,bool,vec,mat,ratvec,LieType,RootDatum,WeylElt,InnerClass,RealForm,CartanClass,KGBElt,Block,Split,KType,KTypePol,Param,ParamPol`. `"void"` is NOT primitive: name maps to the empty tuple in `mk_prim_type` (307-310).

**Tuples/void**: a tuple is a `type_list`; length-1 tuples/unions are forbidden (identified with the component, 351-356); `void` = 0-tuple (`tuple_variant==nullptr` prints `"void"`, 1650-1657).

**Unions**: same `tuple_variant` list under `union_type` tag; the type expression stores NO variant tags. Injector/field names live in the typedef table: `type_binding { id_type name; type_expr type; std::vector<id_type> fields; }` (897-901); `fields` exists mainly so union-controlled `case` can map tags (886-894).

**Tabled/typedefs**: static `type_expr::type_map` (`defined_type_mapping : vector<type_binding>`, 902-911); `expansion()` (973-974); `add_typedefs` canonicalises so equivalent recursive types get one entry — hence tabled equality is just `type_number==` ("no accidental equalities", 715-717).

**`specialise(pattern)`** (708-752): pattern `*` → trivially true; self `*` → `set_from(pattern.copy())`; tabled-vs-tabled compares numbers; tabled-vs-other expands (self side uses `can_specialise` since table types have no holes, 695-706); otherwise tags must match and recurse (functions both directions, row component, tuple/union componentwise 743-752). Result on success = most-general-unifier; explicitly NOT commit-or-rollback (686-693) — `can_specialise` (758-799) exists for when rollback matters.

# 2. convert_expr shape, coercions, balancing (`axis.w`, `axis-types.w`)

`expression_ptr convert_expr(const expr& e, type_expr& type)` (axis.w:272, 478-487): one function does both checking and synthesis. `type` is an in/out pattern — undetermined `*`, partial `(int,*)`, or full; the only permitted mutation is via `specialise` (axis.w:230-241). On success `type` is usually fully determined; `*` remaining means the expression cannot yield a value (die/error, 256-269).

**`coerce(from,to,e)`** (axis-types.w:3062-3074): linear scan of `coerce_table` for exact `from`/`to` match, wraps `e` in a `conversion` node; `to==void` always succeeds with NO node (voiding is the analyser's responsibility). `conform_types(found,required,d,e)` = `required.specialise(found)` else `coerce` else `type_error` (3095-3100). `conversion::evaluate`: eval arg to single value, run `conv_f` on the stack; prints `"name:expr"` (2900-2909).

**Complete coercion table** (registrations global.w:2526-2552, atlas-types.w:9137-9144):
| from → to | tag |
|---|---|
| int→rat | "QI" |
| [int]→vec | "V[I]" |
| [rat]→ratvec | "Qv[Q]" |
| vec→[int] | "[I]V" |
| ratvec→[rat] | "[Q]Qv" |
| vec→ratvec | "QvV" |
| [int]→ratvec | "Qv[I]" |
| vec→[rat] | "[Q]V" |
| [int]→[rat] | "[Q][I]" |
| [vec]→mat | "M[V]" |
| [[int]]→mat | "M[[I]]" |
| [[int]]→[vec] | "[V][[I]]" |
| [vec]→[[int]] | "[[I]][V]" |
| mat→[vec] | "[V]M" |
| mat→[[int]] | "[[I]]M" |
| mat→[ratvec] | "[Qv]M" |
| mat→[[rat]] | "[[Q]]M" |
| [vec]→[ratvec] | "[Qv][V]" |
| [vec]→[[rat]] | "[[Q]][V]" |
| [[int]]→[ratvec] | "[Qv][[I]]" |
| [[int]]→[[rat]] | "[[Q]][[I]]" |
| string→LieType | "LT" |
| InnerClass→RootDatum, RealForm→InnerClass, RealForm→RootDatum | |
| int→Split "SpI", (int,int)→Split "Sp(I,I)" | |
| KType→KTypePol "KpolK", Param→ParamPol "PolP" | |

There is NO string-concat coercion (string `+`/`##` are overloads) and deliberately no coercion producing a tuple type (axis-types.w:2867-2883). `row_coercion(final,comp)` (3120-3128) handles list displays in non-row context: picks the FIRST table entry with row `from`; for `mat` that yields component `vec`, not `[int]` (3113-3117).

**Balancing** (`balance`, axis.w:1022-1048): each branch converted against a fresh copy of `target`; `common` maintained as broadest under `broader_eq`, incomparables set aside in `conflicts`; nested top-level `balance_error`s absorbed (list-display variants wrapped in row-of, 1093-1102); prune conflicts narrower than final `common` (1117-1122); success ⇒ `target.specialise(common)` and RE-convert every branch whose type ≠ target (1043-1047, possibly inserting coercions); else `balance_error`. `broader_eq(a,b)` (axis-types.w:3339-3364): `void` broadest, `*` narrowest; primitive `a` ⇒ `is_close(a,b)&0x2`; rows/tuples componentwise; function types need equal arg types, `broader_eq` on results. Used by list displays (axis.w:1164-1194), conditionals/case; void-row displays get explicit voiding of components (1204-1209).

# 3. Overloaded call resolution (`axis.w`)

Call head dispatch (2396-2439): if `call.fun` is an applied identifier AND not locally bound with function type, fetch `global_overload_table->variants(id)`; if nonempty or `is_special_operator(id)` (`#`, `##`, `## `, `print`, `to_string`, `prints`, `error`, 1777-1813) → `resolve_overload`. Local function bindings shadow ALL overloads; a global Id_table function value is used only when the overload table has no variants (2413-2421). Fallback path: convert fun against `gen_func_type (*->*)`, convert arg against `f_type.func()->arg_type`, build dynamic `call_expression`, `conform_types` on result type (2403-2410).

`resolve_overload` (1552-1581), the post-exponential-blowup design (rationale 1479-1512):
1. Convert each argument ONCE in undetermined context → `a_priori_type` (n>1 args converted componentwise into `unknown_tuple(n)`, 1587-1599).
2. Single pass over variants: exact `a_priori_type==arg_type` → build call now; first variant with `is_close(apt,arg)&0x1` (coercion apt→arg possible) recorded as inexact match, break (1566-1573).
3. Generic/special operators tried next, exact pattern match only (except `print`, which is transparent and may re-convert its arg in the outer context, 1525-1530, 2482-2491): so exact table match beats generic, generic beats inexact table match (1532-1544).
4. Inexact match: per argument whose a-priori component type differs, either directly `coerce` (when the expression form "shields out" context — function calls, assignments, casts, subscriptions, identifiers, denotations, lambdas... full list 1727-1743) or re-run `convert_expr` against the expected component type (1685-1712). Failures here are hard `type_error`s, not failed matches.
5. Else error (1752-1762). No "guess" mechanism survives — the a-priori-type + `is_close` pass replaced the old try-all-variants backtracking.

Extras: `=`-in-void-context error suggesting `:=` (1633-1643); voiding inserted for void-argument functions called with nonempty args (1650-1651); call node built via virtual `function_base::build_call` (builtin_call vs closure_call, 1601-1616). `is_close` 3-bit contract: 0x4 close, 0x1 x→y coercible, 0x2 y→x (axis-types.w:3246-3285).

# 4. Lambda conversion and closures (`axis.w`)

**Lexical layers**: `class layer` (303-344) = vector of (id, type) + constness `BitMap` + `loop_depth` + `return_type*`, self-pushing onto static `layer::lexical_context`. `lookup(id,&depth,&offset,&is_const)` skips EMPTY layers without counting depth (424-442); `layer::specialise(depth,offset,t)` refines a binding post-lookup (462-467).

**Patterns**: `id_pat` kind bits: 0x1 has name, 0x2 has sublist, 0x4 const. `pattern_type` builds `(*,*,(*,*))`-style patterns (2707-2711); `thread_bindings(pat,type,layer,is_const)` flattens ids depth-first into the layer (2743-2754); runtime twin `thread_components(pat,val,out)` flattens a value (tuple forced per sublist) into a frame (2763-2776).

**let**: convert initialiser against `pattern_type(pat)`, then `layer(n)`, `thread_bindings`, convert body (2794-2811); n==0 compiles to `seq_expression` (empty frames are never pushed). Evaluate: eval initialiser single-value; `frame fr(pattern); fr.bind(pop_value()); body->evaluate(l)` (2875-2883).

**lambda**: `case lambda_expr` (3093-3115): check `pattern_type(pat).specialise(arg_type)` (declared parameter type); `type.specialise(gen_func_type)` + `arg_type` into it, `rt=&type.func()->result_type` (or dummy in void context); `layer(count,rt)` marks function boundary (return legality/return type); body converted against `*rt`. Result node `lambda_expression` holds `shared_lambda` → `lambda_struct { id_pat param; expression_ptr body; source_location loc }` (2958-2978), shared with all closures.

**rec_fun**: `case rec_lambda_expr` (3137-3158): result type syntactically mandatory, so `f_type=(arg->result)` is fully determined; layer gets `1+count` ids — self bound const to `f_type` first, then params. Representation trick: pattern = `(self_id, param)` pair via `rec_pair` (3014-3024); `recursive_lambda` derives from `lambda_expression` with no extra data.

**Closures**: `closure_value<kind∈{parameterless,with_parameters,recursive_closure}> { shared_context context; shared_lambda p; const expression_base& body; }` (3209-3236). `lambda_expression::evaluate` captures `frame::current` by shared pointer — NO copying; the environment is a heap-allocated shared singly-linked list (3277-3310). Apply (3517-3560): `lambda_frame` swaps `frame::current` with closure's context plus one new `evaluation_context` node; `bind(pop_value())` threads args (also the interrupt-check point, 3390-3410); recursive closures first push themselves (`maybe_push`/`closure_call` pushes `f`), then `wrap_tuple<2>()` pairs (self,arg) and bind the pair pattern (3548-3560, 3591-3610). `return` = thrown `function_return` caught in apply, value pushed via `push_expanded` (3569-3571). Explicitly "de Bruijn indices" — names unused at runtime (3487-3495).

# 5. Assignment typing (`axis.w`, desugar in `parser.y`/`parsetree.w`)

- **Simple `x:=e`** (`ass_stat`, single-id branch, 7142-7176): lookup local (layer) then global; const → error; convert rhs against a COPY of the identifier's type; if rhs type came out strictly more special, write it back via `layer::specialise`/`global_id_table->specialise` (7157-7163); voiding if variable has void type (7168-7169); emit `local_assignment(depth,offset)` (6929-6954) or `global_assignment(shared_share address)` captured at analysis time (6895-6920). Assignment yields the assigned value (`push_expanded`), `conform_types(rhs_type,type)` at the end.
- **Multi `set (a,b):=e`** (7177, threader at 7322-7393): post-order traversal of the pattern builds the required rhs type by `specialise`-ing each destination's known type into a tuple pattern; repeated ids and `!` qualifiers rejected; destinations recorded as `loc_list` (depth,offset) + `glob_list` (shared_share) + `is_global` BitMap in left-to-right order (6970-6999). Evaluate: eval rhs, `thread_assign` distributes tuple components through `dest_iterator::receive` (7018-7084).
- **Component `x[i]:=e`** (`comp_ass_stat`, 8148-8192): aggregate must be an identifier; index converted freely; `subscr_base::index_kind(aggr_t,ind_t,comp_t)` classifies into `sub_type { row_entry, vector_entry, ratvec_entry, string_char, matrix_entry, matrix_column, K_type_poly_term, mod_poly_term }` (3730-3750); `assignable(kind)` excludes string_char; rhs converted against `comp_t`; `row_entry` case specialises the aggregate's component type (8172-8173); local/global × reversed(`~`) template variants. Copy-on-write whole-value semantics, no sub-objects (7498-7517).
- **Field `x.sel:=e`** (`field_ass_stat`, 8217-8266): selector looked up in `global_overload_table->entry(selector,*tuple_t)` and must be a `projector_value` (exact tuple type match); rhs converted against a modifiable in-place reference to the tuple component type; `local/global_field_assignment` store the projector's `position`.
- **Operate-assign desugar is at PARSE time**: `x op:= e` → `make_assignment(x, make_binary_call(op, x, e))` (parser.y:268-271) — plain assignment of a formula. `v[i] op:= e` and `x.sel op:= e` become dedicated `comp_trans_stat`/`field_trans_stat` nodes (parser.y:272-276; parsetree.w:2934-2967; semantics "let $=I in v[$]:=v[$] op E", parsetree.w:2923-2927), type-checked at axis.w:8495/8295: the operator call is converted normally, then if it resolved to a built-in whose first-operand/result type equals the component type, an in-place `component_transform`/`field_transform` (7566-7615) is built; otherwise it degrades to ordinary component/field assignment of the call result. Related: `x:=f(x)` "hunger" pilfering optimisation (7164-7166, 7179-7188).

# 6. Runtime value model (`axis-types.w`, `global.w`)

`value_base` (axis-types.w:2128-2141): virtual `print`; all values via `shared_ptr<const value_base>`; copy-on-write via `get_own`/`uniquify`.

| kind | payload | print |
|---|---|---|
| `int_value` | `arithmetic::big_int` — ARBITRARY precision (global.w:1870-1876, 1902-1923) | `out<<val` |
| `rat_value` | `big_rat` (1928-1943) | `out<<val` (n/d) |
| `string_value` | `std::string` (1952-1962) | WITH quotes `"..."` (1959) |
| `bool_value` | bool; shared `global_false/true` singletons (1968-1990) | `boolalpha` (`true`/`false`) |
| `vector_value` | `int_Vector = matrix::Vector<int>` — MACHINE 32-bit int entries (2028-2039; Atlas.h:230) | see below |
| `matrix_value` | `int_Matrix` (int entries; iterator ctor fills BY COLUMNS, 2049-2061) | see below |
| `rational_vector_value` | `rat_Vector = RationalVector<Numer_t>`, `Numer_t = long long`, `Denom_t = unsigned long long` (64-bit), normalised in every ctor (2073-2094; arithmetic_fwd.h:21-24) | see below |
| `row_value` | `vector<shared_value>` (axis-types.w:2187-2201) | `[a,b,c]` — comma, NO padding (2209-2214) |
| `tuple_value` | derives row_value (2226-2237) | `(a,b,c)` (2241-2246) |
| `union_value` | `{shared_value comp; unsigned short tag; id_type injector_name}` (2318-2334) | `value.injectorname` (2341-2342) |
| `builtin_value<variadic>` | wrapper fn ptr + print_name + hunger (axis.w:1975-2002) | `{name}` |
| `closure_value<kind>` | context + lambda (axis.w:3211-3236) | `Function defined <loc>\n` + lambda (3254-3271) |

**Exact vec/ratvec/mat printing** (differential-critical, global.w:2107-2158):
- `vec`: compute max entry width w, then w+=1; `[` then each entry right-aligned in `setw(w)`, `,` between, last entry followed by `" ]"` (space before `]`); empty → `"[ ]"`. E.g. `[  1, 22 ]` pattern.
- `ratvec`: identical fields over NUMERATORS, then `'/' << denominator` after the `]` (2122-2135).
- `mat`: if rows==0 or cols==0 → `"The KxL matrix"` (no newline); else a LEADING `std::endl`, then per row: `'|'`, each entry right-aligned in `setw(w[j]+1)` (per-COLUMN width), `,` between columns, `' '` after last, `'|'`, `std::endl` (2141-2158).
- `conversion` prints `name:expr`; `voiding` prints `voided:expr` (axis-types.w:2908-2909, 3034-3035).

Type printing (axis-types.w:1610-1675): `*`; prim names; `[comp]`; `(a,b)`; `(a|b)`; functions `(arg->res)` with naked tuple/union arg/result lists; empty tuple `void`; tabled by name else expansion.

# 7. Frames and evaluation discipline

- `expression_base::evaluate(level l)`, `level { no_value, single_value, multi_value }`, wrappers `void_eval/eval/multi_eval` (axis-types.w:2424-2440). One global `execution_stack` of `shared_value` (2474-2484); `push_expanded` expands a tuple onto the stack only at `multi_value` (with a unique-ownership move fast path) (2506-2534).
- **Environment**: `evaluation_context { shared_context next; vector<shared_value> frame; }` — heap shared linked list; `elem(i,j)` walks `i` links then indexes `j` (2370-2400). `frame::current` is a static `shared_context` (axis.w:1358-1359); `frame` ctor pushes a node by MOVING current into the tail, dtor pops by COPY (closures may share the tail) (2830-2848). Empty layers correspond to NO frame and don't count toward depth (414-421, 2785-2792).
- **Identifiers**: locals are `(depth,offset)` fixed at analysis (1370-1398); globals capture the `shared_share` address at analysis time, so re-`set`ting a global with a new type never affects already-converted code (1258-1323); reading an uninitialised global is a runtime error (1314-1318). `pilfer` template variants move the value out (for the x:=f(x) optimisation).
- **Calls**: builtins take args EXPANDED (`argument_policy()==multi_value`), variadic builtins and closures take one value (`single_value`) (axis.w:1988-1990, 3222-3223). Dynamic `call_expression::evaluate`: eval fun, `dynamic_pointer_cast<function_base>`, `maybe_push`, eval args at policy, `apply(l)` in a try block that appends call trace-back (2340-2368).
- **Strictness**: fully eager call-by-value; the only laziness-like machinery is the `level` protocol — void contexts propagate `no_value` downward, and analysis guarantees any void-typed subexpression is evaluated with `no_value` (explicit `voiding` nodes inserted only where syntax doesn't imply it: rhs of void-typed assignment, void tuple/list components, void-arg calls) (axis-types.w:2990-3059). `voiding::evaluate` runs `void_eval` and pushes `()` only if `single_value` was demanded (3028-3032).

---

All data gathered. Here is the structured inventory.

# ATLAS STARTUP BUILTIN INVENTORY (oracle: atlasofliegroups master 4d3e9449)

Notation: `g:N` = `sources/interpreter/global.w:N`, `a:N` = `atlas-types.w:N`, `x:N` = `axis.w:N`, `m:N` = `main.w:N`. Cites are `install-site / impl-site` (impl = `void <wrapper>_wrapper`). Totals: **164 installs in global.w, 305 in atlas-types.w**, plus the axis.w generics below. `install_function` is declared g:2575, defined g:2591; it builds the print-name `name@arg_type` and enters the overload table (void-argument builtins go to `global_id_table` instead, g:2588–2599). Trailing `unsigned char hunger=0` per overload is the coercion-tolerance mask used by overload matching (already covered in your trace doc).

## S. SPECIAL (not plain overload-table functions)

- **Generic operators resolved by the type-checker, not the overload table.** `is_special_operator` (x:1806–1813) hard-codes seven names: `#`, `##`, `## ` (trailing space, "protected concatenate"), `print`, `to_string`, `prints`, `error` (id lookups x:1777–1804). When no specific overload matches, generic fallback runs at x:6765. Static builtin objects x:8750–8789:
  - `#@[T]` (`sizeof_wrapper`, x:8868) — length of any row value.
  - `#@(T,[T])` prefix / `#@([T],T)` suffix (`prefix_element_wrapper`/`suffix_element_wrapper`, x:8893; suffix is amortised O(1), enabling `#:=` accumulation).
  - `##@([T],[T])` join two rows; `##@([[T]])` flatten row of rows (x:8782–8787).
  - `print@T` (x:8804) — prints value in interpreter format, **returns its argument unchanged** (pass-through for debugging).
  - `to_string@T` (x:8843) — like prints-formatting into a string: strings lose quotes, tuple components concatenated unseparated (helper `to_string_aux` x:8820).
  - `prints@T` (x:8850) — `to_string` formatting + newline to output stream, returns void.
  - `error@T` (x:8856) — same formatting, then **throws `runtime_error`** with that message (this is how scripts raise; caught by `die`-level machinery and produces back_trace).
  - `not@bool` (x:8788) — used for negation_expr; `not` itself is a keyword (m:262).
  Note the type-checker also short-circuits `=` (equals_name x:1815) and the specific `#@vec/#@ratvec/#@string/#@mat/#@ParamPol` builtins are re-bound as statics x:8752–8766 so generic `#` can defer to them.
- **`install_special_function`** (g:2578, g:2616): returns a `special_builtin` whose `build_call` does compile-time folding. Used only for: unary `-@(int)` (g:2966), `+@(int,int)` (g:2971), `-@(int,int)` (g:2978), `/@(int,int)->rat` (g:3231). Paired with `succ`/`pred` install (g:2969/2974) so e.g. `n+1` becomes `succ`; negated integer denotations fold (doc g:2903–2965).
- **`$`** — last value computed; parses to `expr::last_value_computed` (x:588–600); also a hidden identifier `$` used internally for component-assignment rewriting (x:8721).
- **`back_trace`** — NOT a function: a global variable of type `[string]`, registered in `global_id_table` at m:430–434; filled by `set_back_trace(err.back_trace)` when a runtime error reaches top level (g:1127, g:1135–1146). basic.at's `where()` reads it (basic.at:10–13).
- **`break` / `return` / `die`** — keywords (m:265), not builtins: `break`/`return` are expressions implemented by throwing `loop_break` (axis-types.w:3531) / `function_return` (axis-types.w:3544); `die` is `die_expr` (parsetree.w:519, x:637) which throws unconditionally.
- **System variables** at startup besides back_trace: `input_path: [string]` (m:414), `prelude_log: [string]` (m:423); identifiers `quiet`, `verbose` reserved first (m:283).
- **Space-suffixed alias names** (uninvokable directly, used by internal rewrites): `## @(string,string)`-style protected concat (x:1785), `transpose @(mat->mat)` (g:5188) alias of `^@mat`, `matrix slicer` (g:5197) alias of `swiss_matrix_knife`.
- **`elapsed_ms@(->int)`** (g:5245) — void-argument builtin, so it lives in the identifier table, not the overload table.

## 1. BUILTINS WHOSE NAME APPEARS IN basic.at (grep), exact signatures + semantics

### 1a. int / bitset (installs g:2966–2994)
| name | sigs | semantic | cite |
|---|---|---|---|
| `-` | `(int->int)`, `(int,int->int)` | negate; subtract (special/folding) | g:2966,2978 |
| `+` | `(int,int->int)` | add (special/folding) | g:2971/g:2637 area |
| `*` | `(int,int->int)` | multiply | g:2982/g:2675 |
| `\` | `(int,int->int)` | floor-quotient (Euclidean: rounds toward −∞) | g:2983/g:2712 |
| `%` | `(int,int->int)` | non-negative remainder of `\` | g:2984/g:2732 |
| `\%` | `(int,int->int,int)` | (quotient, remainder) pair | g:2985/g:2743 |
| `^` | `(int,int->int)` | integer power | g:2986/g:2792 |
| `~` | `(int->int)` | bitwise complement (−1−n) | g:2976 |
| `AND` `OR` `XOR` `AND_NOT` | `(int,int->int)` | bitwise ops on infinite-2-adic int-as-bitset | g:2987–2990/g:2817–2841 |
| `bitwise_subset` | `(int,int->bool)` | bits(a) ⊆ bits(b) | g:2991/g:2852 |
| `bit_length` | `(int->int)` | number of bits up to highest set | g:2993/g:2872 |
| `to_bitset` | `(vec->int)` | int with bits at positions listed in v | g:2994/g:2887 |
| (`pred`,`succ`,`nth_set_bit` also installed here; `pred` grep-hit in basic.at is a local identifier only — see §3) |

### 1b. rat (g:3229–3252)
`/@(int->rat)` inverse; `/@(int,int->rat)` fraction (special, normalises); `%@(rat->int,int)` = (numer,denom) decomposition — basic.at's `%a` idiom; `+ - * /  %` each at `(rat,int->rat)` and `(rat,rat->rat)` (g:3236–3246); `\@(rat,int->int)` floor-quotient (g:3240); `-@(rat->rat)`, `/@(rat->rat)` inverse (g:3247–3248); `floor@(rat->int)` g:3249/g:3153, `ceil` g:3250/g:3158, `frac@(rat->rat)` fractional part g:3251/g:3163; `^@(rat,int->rat)` g:3252.

### 1c. comparisons/equality (g:4352–4420)
Unary sign tests against zero and binary comparisons: `= != >= > <= <` at `(int->bool)` and `(int,int->bool)` (g:4352–4363); same 12 for rat (g:4364–4375); `= !=` at `(bool,bool->bool)` (g:4376–4377); `= != < <= > >=` for string unary/binary (g:4378–4385); `= !=` unary+binary and `>= >` unary (all/any-entry positivity: `>=v` = all entries ≥0, `>v` = nonzero and ≥0 — dominance tests) for vec (g:4405–4410) and ratvec (g:4411–4416); `= !=` unary+binary for mat (g:4417–4420). Unary `=x` means "x is zero/empty"; `!=x` nonzero.

### 1d. string (g:4386–4392)
`##@(string,string->string)` concat (g:4386); `##@([string]->string)` concat list (g:4387); `ascii@(string->int)` first-char code (g:4388/g:3516); `ascii@(int->string)` char (g:4389/g:3523). (`readline_completions@(string->[string])` g:4390 — not in basic.at.)

### 1e. sizes, vec/ratvec/mat structure (g:4392–4404, 5183–5211)
`#` at `(string->int)`, `(vec->int)`, `(ratvec->int)`, `(mat->int)`(=n_columns) g:4392–4395; `#@(vec,int->vec)` suffix entry, `#@(int,vec->vec)` prefix g:4396–4397; `##@(vec,vec->vec)`, `##@([vec]->vec)` g:4398–4399; `shape@(mat->int,int)` (rows,cols) g:4400/g:3610; `row@(mat,int->vec)`, `column@(mat,int->vec)` g:4401–4402; `rows@(mat->[vec])`, `columns@(mat->[vec])` g:4403–4404/g:2432,2441; `null@(int->vec)` zero vector, `null@(int,int->mat)` zero matrix g:5183–5184; `^@(vec->mat)` single-row matrix (transpose of column), `^@(mat->mat)` transpose g:5185–5186; `id_mat@(int->int→mat)` identity g:5190/g:4527; `#@(int,[vec]->mat)` assemble columns with forced row-count n, `^@(int,[vec]->mat)` assemble rows g:5193–5194/g:4603,4632; `swiss_matrix_knife@(int,mat,int,int,int,int->mat)` — 8-bit flag slicer: bits 0–2 row slice reverse/lwb-negate/upb-negate, bits 3–5 same for columns, bit 6 transpose, bit 7 negate entries (doc g:4705–4711, impl g:4714). basic.at leans on it for `~@mat`(45), `-@mat`(164), `negative_system`(172).

### 1f. vec/ratvec/mat arithmetic (g:4421–4451)
vec: `+ -`@(vec,vec), unary `-`, `*@(vec,int)`, `\@(vec,int)`, `%@(vec,int)` entrywise; `/@(vec,int->ratvec)`; `*@(vec,vec->int)` dot product (g:4441). ratvec: `%@(ratvec->vec,int)` = (numer-vec, common denom) — basic.at's `%v`; `+ -` binary/unary; `* /  %` by int; `* /` by rat (g:4428–4436). mat: `+ -`@(mat,int),(int,mat) scalar-on-diagonal add (m±k·Id), `+ -`@(mat,mat); products `*` at `(mat,ratvec->ratvec)`, `(mat,vec->vec)`, `(mat,mat->mat)`, `(vec,mat->vec)`, `(ratvec,mat->ratvec)` (g:4447–4451).

### 1g. linear algebra (g:5200–5211)
`gcd@(vec->int)` gcd of entries (g:5200/g:4820); `echelon@(mat->mat,mat,[int],int)` — column echelon: (M,C,pivot_rows,det_sign_flip) with A·C=M (g:5202/g:4848); `linear_solve@(mat,vec->|vec,int,mat)` — **union result** `(void empty_set | (vec,int,mat) affine_subspace)`, matching basic.at's `set_type linear_solution` (g:5203/g:4891); `kernel@(mat->mat)` basis of integer kernel (g:5206/g:4975); `Smith@(mat->mat,vec)` (adapted basis, invariant factors) (g:5209/g:5000); `invert@(mat->mat,int)` inverse×d with denominator d, d=0 if singular (g:5210/g:5017). (Also installed here, names not in basic.at: `Bezout`, `diagonalize`, `adapted_basis`, `eigen_lattice`, `row_saturate`, `mod2_section`, `subspace_normal`, `stack_rows`, `diagonal`, `flex_add`, `flex_sub`, `convolve` — see §2.)

### 1h. LieType layer (a:423–436, 932–941)
`Lie_type@(string->LieType)` parse "A2.T1" style (a:423/a:229); `*@(LieType,LieType->LieType)` concat (a:424); `extend@(LieType,string,int->LieType)` append one factor (a:425/a:280); `= !=@(LieType,LieType->bool)` (a:427–428); `Cartan_matrix@(LieType->mat)` (a:430/a:319); `Cartan_matrix_type@(mat->LieType,[int])` recognise type + permutation to standard numbering (a:431); `simple_factors@(LieType->[string,int])` (a:434/a:388); `rank@(LieType->int)` (a:436); `quotient_basis@(LieType,[ratvec]->mat)` lattice basis for quotient by kernel gens (a:937/a:655); `involution@(LieType,[int],string->mat)` and `@(LieType,mat,string->mat)` distinguished involution from inner-class letters (a:939,941/a:860,902). (`Smith_Cartan`, `filter_units`, `ann_mod`, `replace_gen`, `is_Cartan_matrix` also installed a:932–935,433 — §2.)

### 1i. RootDatum layer (a:2203–2290)
- Constructors: `root_datum@(mat,mat,bool->RootDatum)` from simple (co)root columns + prefer-coroots flag (a:2207/a:1230); `root_datum@(LieType,mat,bool)` type+sublattice-basis (a:2209/a:1270); `root_datum@(RootDatum,mat)` sublattice (a:2211, name-hit only); `simply_connected@(LieType,bool)` (a:2213/a:1331); `adjoint@(LieType,bool)` (a:2215/a:1346).
- Attributes: `Lie_type@(RootDatum->LieType)` a:2203; `prefers_coroots@(RootDatum->bool)` a:2205; `= !=` a:2218–2219; `Cartan_matrix@(RootDatum->mat)` a:2220; `rank`, `semisimple_rank`, `nr_of_posroots` `(RootDatum->int)` a:2221–2224; `two_rho`, `two_rho_check` `(RootDatum->vec)` a:2225–2226.
- Roots: `root@(RootDatum,int->vec)` / `coroot` — index i∈[−#posroots,#posroots), negatives via `~i` for negative roots (a:2228–2229/a:1450,1458); `root_index@(RootDatum,vec->int)` / `coroot_index` — inverse lookup, returns #posroots for non-roots, negative (~) for negative roots (a:2230–2231/a:1487,1495); `root_expression@(RootDatum,int->vec)` / `coroot_expression` — coordinates on simple (co)roots (a:2232–2234/a:1507,1515); `root_ladder_bottoms@(RootDatum,int->[int])` / `coroot_ladder_bottoms` (a:2241–2243/a:1569,1584) — root-string bottom indices used by basic.at highest-root machinery; `fundamental_weight@(RootDatum,int->ratvec)` / `fundamental_coweight` (a:2245–2247/a:1605,1618); `simple_roots`,`simple_coroots`,`posroots`,`poscoroots` `(RootDatum->mat)` columns (a:2250–2253/a:1638–1665); `root_coradical` / `coroot_radical` `(RootDatum->mat)` — simple (co)roots extended by (co)radical basis (a:2254–2255/a:1679,1691).
- Derived data: `dual@(RootDatum->RootDatum)` a:2257/a:1713; `derived_info@(RootDatum->RootDatum,mat)` derived group + projection (a:2258/a:1719); `mod_central_torus_info@(RootDatum->RootDatum,mat)` (a:2260/a:1732); `integrality_datum@(RootDatum,ratvec->RootDatum)` subdatum of coroots integral on γ (a:2262/a:1749); `integrality_rank@(RootDatum,ratvec->int)` a:2264/a:1768; `is_integrally_dominant@(RootDatum,ratvec->bool)` a:2266/a:1783.

### 1j. WeylElt layer (a:2629–2649)
`W_elt@(RootDatum,[int]->WeylElt)` from word (a:2629/a:2361); `word@(WeylElt->[int])` reduced word (a:2630/a:2374); `root_datum@(WeylElt->RootDatum)` a:2631; `length@(WeylElt->int)` a:2632; `= !=` unary(=identity test)/binary a:2633–2636; `*@(WeylElt,WeylElt)` a:2637; `/@(WeylElt->WeylElt)` inverse a:2638; `#@(WeylElt,int)`/`#@(int,WeylElt)` multiply by one generator a:2639–2640; `##@(WeylElt,[int])`/`##@([int],WeylElt)` multiply by word a:2641–2642; `*@(WeylElt,vec->vec)` act on weight, `*@(vec,WeylElt->vec)` coweight acts from right a:2643–2644; `from_dominant@(RootDatum,vec->WeylElt,vec)` — (w, dominant v0) with v = w·v0; coweight version `from_dominant@(vec,RootDatum->vec,WeylElt)` with reversed order/interpretation (a:2645–2647/a:2561,2580, doc a:2552–2559).

### 1k. InnerClass (a:3400–3429)
`inner_class@(RootDatum,mat->InnerClass)` from involution matrix (a:3402/a:3186); `= !=` a:3406–3407; `distinguished_involution@(InnerClass->mat)` a:3408/a:3236; `root_datum@(InnerClass->RootDatum)` a:3410/a:3242; `dual@(InnerClass->InnerClass)` a:3414/a:3256; `form_names@(InnerClass->[string])` a:3416/a:3277 (and `dual_form_names` a:3417); `nr_of_real_forms` a:3419/a:3294; `nr_of_Cartan_classes@(InnerClass->int)` a:3423/a:3306; `block_sizes@(InnerClass->mat)` forms×dual-forms size table a:3425/a:3323.

### 1l. RealForm (a:3930–3953)
`real_form@(InnerClass,int->RealForm)` by weak-form number (a:3930/a:3585); `form_number@(RealForm->int)` a:3931/a:3598; `quasisplit_form@(InnerClass->RealForm)` a:3932/a:3605; `inner_class@(RealForm->InnerClass)` a:3934/a:3617; `nr_of_Cartan_classes@(RealForm->int)` a:3937/a:3645; `KGB_size@(RealForm->int)` a:3939/a:3655; `KGB_Hasse@(RealForm->mat)` closure order Hasse a:3943/a:3735; `= !=@(RealForm,RealForm->bool)` a:3944–3945; `dual_quasisplit_form@(InnerClass->RealForm)` a:3948/a:3809; `real_form@(InnerClass,mat,ratvec->RealForm)` synthetic form from (involution θ, torus factor) (a:3950/a:3851).

### 1m. CartanClass (a:4347–4363)
`Cartan_class@(InnerClass,int)` and `@(RealForm,int)` (a:4347–4349/a:4019,4040); `most_split_Cartan@(RealForm->CartanClass)` a:4351/a:4065; `involution@(CartanClass->mat)` canonical θ a:4353/a:4080; `Cartan_info@(CartanClass->(int,int,int),vec,(int,int),(LieType,LieType,LieType))` = ((compact,Complex,split ranks), canonical twisted-involution word, (orbit size, fiber size), (imaginary,real,complex types)) (a:4354/a:4102) — basic.at destructures exactly this; `real_forms@(CartanClass->[RealForm])` / `dual_real_forms` — forms containing this Cartan (a:4357–4359/a:4155,4171).

### 1n. KGB (a:4702–4717)
`KGB@(RealForm,int->KGBElt)` a:4702/a:4412; `%@(KGBElt->RealForm,int)` decompose a:4703/a:4429; `cross@(int,KGBElt->KGBElt)` a:4704/a:4490; `Cayley@(int,KGBElt->KGBElt)` (no-op if undefined) a:4705/a:4506; `status@(int,KGBElt->int)` codes 0=C−,1=ic,2=r,3=nc,4=C+ (descent iff <3, imaginary iff odd, Cayley defined iff 3 — doc a:4530–4537, impl a:4539); `KGB_elt@(RealForm,mat,ratvec->KGBElt)` from (θ, torus factor) a:4707/a:4580; `Cartan_class@(KGBElt->CartanClass)` a:4711/a:4444; `involution@(KGBElt->mat)` a:4712/a:4452; `length@(KGBElt->int)` a:4713/a:4461; `torus_factor@(KGBElt->ratvec)` a:4715/a:4670; `= !=` a:4716–4717. (`twist`, `torus_bits`, `initial_torus_bits`, `base_grading_vector`, `Cartan_order`, `central_fiber`, `components_rank`, `count_Cartans` = `nr_of_Cartan_classes@RealForm` — §2.)

### 1o. Block (a:4995–5004)
`block@(RealForm,RealForm->Block)` Fokko block (a:4995/a:4786); `#@(Block->int)` size a:4997/a:4820; `dual@(Block->Block)` a:5000; `status@(int,Block,int->int)`, `cross@(int,Block,int->int)`, `Cayley@(int,Block,int->int)` a:5001–5003. (`%@Block`, `element`, `index`, `inverse_Cayley` — names `element`/`index` hit basic.at only in comments, see §3.)

### 1p. Split (a:5137–5145)
`= !=` unary/binary; `+ -@(Split,Split)`, unary `-`, `*@(Split,Split)`; `%@(Split->int,int)` decompose a+bs → (a,b) — basic.at's `%x` on Split (a:5137–5145).

### 1q. KType / KTypePol (a:6071–6123)
`K_type@(KGBElt,vec->KType)` from (x, λ−ρ) (a:6071/a:5240); `%@(KType->KGBElt,vec)` a:6072/a:5266; `real_form@(KType->RealForm)` a:6073; `height@(KType->int)` a:6075/a:5291; `= !=` a:6076–6077; `is_final@(KType->bool)` a:6083; `null_K_module@(RealForm->KTypePol)` a:6091/a:5537; `real_form@(KTypePol->RealForm)` a:6092; `= !=` unary/binary a:6094–6097; `#@(KTypePol->int)` #terms a:6098; `+ -@(KTypePol,KType)` a:6099–6100; `+@(KTypePol,(Split,KType))` a:6101; `+@(KTypePol,[(Split,KType)])` a:6103/a:5741; `+ -@(KTypePol,KTypePol)` a:6105–6107; `*@(int,KTypePol)`, `*@(Split,KTypePol)` a:6109–6111; `last_term`/`first_term@(KTypePol->Split,KType)` a:6113–6115; `truncate_above_height@(KTypePol,int->KTypePol)` a:6117; `K_type_formula@(KType,int->KTypePol)` char formula with height bound (−1 = none) a:6121/a:6030; `branch@(KTypePol,int->KTypePol)` K-type decomposition up to height a:6123/a:6055. (also `is_standard/is_dominant/is_zero/is_semifinal/equivalent/dominant/normal/to_canonical_fiber/theta_stable/KGP_sum` — §2, though `is_dominant`/`dominant` names do grep-hit basic.at via its own definitions.)

### 1r. Param / ParamPol (a:7472–7533, 8542–8595)
`param@(KGBElt,vec,ratvec->Param)` from (x, λ−ρ, ν) (a:7472/a:6215); `%@(Param->KGBElt,vec,ratvec)` gives (x, λ−ρ, γ) (a:7474/a:6252); `real_form@(Param->RealForm)` a:7476; `height@(Param->int)` a:7478/a:6300; `K_type@(Param->KType)` restriction a:7479/a:6275; `param@(KType->Param)` ν=0 lift a:7480/a:6283; `= !=`, `is_final@(Param->bool)` a:7481–7489; `cross@(int,Param)`, `Cayley@(int,Param)`, `cross@(vec,Param)`, `Cayley@(vec,Param)` — int indexes integrality-datum simples, vec gives a root (a:7492–7495/a:6430,6445); `orientation_nr@(Param->int)` a:7499/a:6549; `reducibility_points@(Param->[rat])` ν-scalings in (0,1] where reducible a:7500/a:6561; `*@(Param,rat->Param)` scale ν a:7502/a:6582; `block@(Param->[Param],int)` common block + index of p (a:7510/a:6748); `length@(Param->int)` a:7513/a:6818; `extended_block@(Param,mat->[Param],mat,mat,mat)` δ-fixed block: (params, type codes, links0, links1) a:7531/a:7366; `print_block@(Param->)` a:7504.
ParamPol: `null_module@(RealForm->ParamPol)` a:8542/a:7613; `real_form@(ParamPol->RealForm)` a:8543; `#@(ParamPol->int)` a:8545; `K_type_pol@(ParamPol->KTypePol)` restrict to K a:8546/a:7717; `= !=` unary/binary a:8548–8551; `+ -@(ParamPol,Param)` a:8552–8553; `+@(ParamPol,(Split,Param))` a:8554; `+@(ParamPol,[(Split,Param)])` a:8556; `+ -@(ParamPol,ParamPol)` a:8558–8560; `*@(int,ParamPol)`, `*@(Split,ParamPol)` a:8562–8564; `last_term`/`first_term@(ParamPol->Split,Param)` a:8566–8567; `truncate_above_height@(ParamPol,int->ParamPol)` a:8568; `*@(ParamPol,rat->ParamPol)` a:8570; `block_deform@(Param,ParamPol,int->ParamPol,ParamPol)` deform p's whole block within P up to height (a:8574/a:8178); `full_deform@(Param->KTypePol)` full deformation to ν=0 (a:8576/a:8213; timed union variant `full_deform@(Param,int->|KTypePol)` a:8579 — result is a union, `|` in signature).

### 1s. KL / printing (a:9101–9129)
`raw_KL@(Block->mat,[vec],vec)` (index matrix, KL polys as coef-vecs, involution permutation) a:9101; `dual_KL@(Block->...)` a:9102; `print_KGB@(RealForm->)` a:9119/a:8944; `print_block/print_blocku/print_blockd@(Block->)` a:9114–9116/a:8869,8894,8883; `print_KL_basis`,`print_prim_KL`,`print_KL_list`,`print_W_cells`,`print_W_graph@(Block->)` a:9125–9129/a:9017–9085.

## 2. REMAINING STARTUP BUILTINS (names + signatures only)

**int/bitset/util (global.w):** `succ(int->int)`, `pred(int->int)`, `nth_set_bit(int,int->int)`, `readline_completions(string->[string])`, `elapsed_ms(->int)`.
**vec/mat extras:** `flex_add(vec,vec->vec)`, `flex_sub(vec,vec->vec)`, `convolve(vec,vec->vec)`, `stack_rows([vec]->mat)`, `diagonal(vec->mat)`, `Bezout(vec->int,mat)`, `diagonalize(mat->vec,mat,mat)`, `adapted_basis(mat->mat,vec)`, `eigen_lattice(mat,int->mat)`, `row_saturate(mat->mat)`, `mod2_section(mat->mat)`, `subspace_normal(mat->mat,mat,mat,[int])`.
**LieType:** `is_Cartan_matrix(mat->bool)`, `Smith_Cartan(LieType->mat,vec)`, `filter_units(mat,vec->mat,vec)`, `ann_mod(mat,int->mat)`, `replace_gen((mat,vec),mat->mat)`.
**RootDatum:** `is_long_root(RootDatum,int->bool)`, `is_long_coroot(RootDatum,int->bool)`, `root_involution(RootDatum,int->vec)`, `integrality_points(RootDatum,ratvec->[rat])`, `Weyl_orbit(RootDatum,vec->mat)`, `Weyl_orbit(vec,RootDatum->mat)`, `Weyl_orbit_ws(RootDatum,vec->[WeylElt])`, `Weyl_orbit_ws(vec,RootDatum->[WeylElt])`, `cofolded(InnerClass->RootDatum)`, `walls(RootDatum,ratvec->[int],int)`, `walls_attitude(RootDatum,[int]->WeylElt)`, `alcove_center(Param->Param)`, `alcove_root_vertex(RootDatum,ratvec->vec)`, `basic_orbit_ws(RootDatum,[int],int->[WeylElt])`, `affine_orbit_ws(RootDatum,ratvec->[WeylElt])`, `FPP_numers(RootDatum,ratvec->[vec])`, `FPP_w_shifts(RootDatum,ratvec->[WeylElt,[vec]])`.
**WeylElt:** `root_permutation(WeylElt->vec)`.
**InnerClass:** `classify_involution(mat->int,int,int)`, `twisted_involution(RootDatum,mat->WeylElt,InnerClass)`, `dual_datum(InnerClass->RootDatum)`, `dual_form_names(InnerClass->[string])`, `nr_of_dual_real_forms(InnerClass->int)`, `block_size(InnerClass,int,int->int)`, `occurrence_matrix(InnerClass->mat)`, `dual_occurrence_matrix(InnerClass->mat)`, `dual_real_form(InnerClass,int->RealForm)`.
**RealForm:** `components_rank(RealForm->int)`, `base_grading_vector(RealForm->ratvec)`, `Cartan_order(RealForm->mat)`, `central_fiber(RealForm->[vec])`, `initial_torus_bits(RealForm->vec)`.
**CartanClass:** `fiber_partition(CartanClass,RealForm->[int])`, `square_classes(CartanClass->[[int]])`.
**KGB:** `twist(KGBElt->KGBElt)`, `twist(KGBElt,mat->KGBElt)`, `torus_bits(KGBElt->vec)`.
**Block:** `%(Block->RealForm,RealForm)`, `element(Block,int->KGBElt,KGBElt)`, `index(Block,KGBElt,KGBElt->int)`, `inverse_Cayley(int,Block,int->int)`.
**KType:** `equivalent(KType,KType->bool)`, `is_standard/is_dominant/is_zero/is_semifinal(KType->bool)`, `dominant/normal/to_canonical_fiber/theta_stable(KType->KType)`, `KGP_sum(KType->[int,KType])`.
**Param:** `equivalent(Param,Param->bool)`, `is_standard/is_dominant/is_zero/is_semifinal(Param->bool)`, `dominant/normal(Param->Param)`, `twist(Param->Param)`, `twist(Param,mat->Param)`, `print_common_block(Param->)`, `print_partial_block(Param->)`, `print_partial_common_block(Param->)`, `partial_block(Param->[Param])`, `block_Hasse(Param->[Param],mat)`, `KL_block(Param->[Param],int,mat,[vec])`, `dual_KL_block(Param->[Param],int,mat,[vec])`, `partial_KL_block(Param->[Param],mat,[vec])`, `W_graph(Param->int,[[int],[int,int]])`, `W_cells(Param->int,[[int],[[int],[int,int]]])`, `strong_components([[int]]->[[int]],[[int]])`, `default_extended(Param,mat->vec,vec,vec,vec)`, `shift_flip(Param,mat,ratvec->bool)`, `partial_extended_KL_block(Param,mat->[Param],mat,[vec])`.
**ParamPol/deformation/KL:** `deform(Param->ParamPol)`, `twisted_deform(Param->ParamPol)`, `twisted_full_deform(Param->KTypePol)`, `twisted_full_deform(Param,int->|KTypePol)`, `KL_sum_at_s(Param->ParamPol)`, `KL_sum_at_s_to_height(Param,int->ParamPol)`, `twisted_KL_sum_at_s(Param->ParamPol)`, `twisted_KL_sum_at_s(Param,mat->ParamPol)`, `KL_column(Param->[int,Param,vec])`, `scale_extended(Param,mat,rat->Param,bool)`, `K_type_pol_extended(Param,mat->KTypePol)`, `finalize_extended(Param,mat->ParamPol)`, `raw_ext_KL(Param,mat->mat,[vec],vec)`, `W_graph(Block->[[int],[int,int]])`, `W_cells(Block->[[int],[[int],[int,int]]])`.
**Printers:** `print_gradings(CartanClass,RealForm->)`, `print_real_Weyl(RealForm,CartanClass->)`, `print_strong_real(CartanClass->)`, `print_blockstabilizer(Block,CartanClass->)`, `print_KGB(RealForm,[KGBElt]->)`, `print_KGB_order(RealForm->)`, `print_KGB_graph(RealForm->)`, `print_X(InnerClass->)`.

## 3. GREP-HIT CAVEATS (name appears in basic.at but builtin is not actually called)

- `pred` (basic.at:80,1219), `index`/`element` (comments only), `walls` (comment basic.at:1061), `linear_solve` (comment basic.at:1 — but its union result type IS declared there, so the builtin is load-bearing for scripts that use it), `transpose ` (comment basic.at:1080), `XOR@(int,int)` (basic.at only defines its own `XOR@[bool]`), `dominant`/`is_dominant` builtins @KType/@Param (basic.at defines its own `dominant`/`is_dominant` over RootDatum via `from_dominant`; the @Param builtins are unused there). Conversely note real but easy-to-miss uses: `row@(mat,int)` (basic.at:1645), `null@(int,int)` (basic.at:2134), `branch@(KTypePol,int)` (basic.at:2046), `block_deform` (basic.at:2064), synthetic `real_form@(InnerClass,mat,ratvec)` (basic.at:1502).

**Load-bearing minimum for basic.at to type-check:** every §1 entry plus the generic §S operators (`# ## print prints to_string error not`), the `%`-decomposition family (rat, ratvec, Split, KGBElt, KType, Param), union-typed `linear_solve`, and global variable `back_trace` (basic.at:11 iterates it). basic.at also requires language features already in your trace (operator aliasing `set ^ = !=@(bool,bool)`, `f@type` overload projection, `set_type` unions, `rec_fun`, `break`/`return`).

Full machine-extracted table (name / signature / install file:line / wrapper / SPECIAL flag) persisted at `/Users/hoxide/.claude/projects/-Users-hoxide-mycodes-atlas-rust/8ac5aee8-31aa-4f1e-879a-466b0bdce8f4/tool-results/bk7152jot.txt`.
