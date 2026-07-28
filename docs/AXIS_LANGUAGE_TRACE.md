# Axis-language oracle trace (upstream master 4d3e9449)

Raw trace notes for the language-compatibility goal: include/lexer
machinery, the full parser.y grammar, definition semantics and the
session output surface. Design docs cite these.

---

All facts verified against source. Final notes follow.

# Oracle notes: atlas INPUT/INCLUDE machinery + lexer (atlasofliegroups @ 4d3e9449, sources/interpreter/)

## 1. Include/redirect directive syntax

Recognized **only when `<`/`>` is the first token of a command** (`state==initial`, lexer.w:554-562). Otherwise `<` `>` are comparison operators.

| Directive | Token | Semantics |
|---|---|---|
| `<file` | FROMFILE | include, skip if already completely read (`include_file(1)`, parser.y:182) |
| `<<file` | FORCEFROMFILE | forced re-include (`include_file(0)`, parser.y:183) |
| `>file expr` | TOFILE | redirect output of one command, truncate (`*verbosity=2`, parser.y:180; opened `ios_base::trunc` main.w:506-511, reset to stdout after the command main.w:520) |
| `>>file expr` | ADDTOFILE | same, append (`*verbosity=3`, parser.y:181, main.w:507) |

- Grammar requires the include to be an entire command: `FROMFILE '\n'` / `FORCEFROMFILE '\n'` (parser.y:182-183) — nothing else on the line (comments OK, they're skipped as space). `>file`/`>>file` prefix a single expr command; also `>file whattype id?` and `>file showall` forms (parser.y:189-217).
- **Filename scanning** (lexer.w:732-740): after skipping space, either a quoted string (`"..."`), or an unquoted run of `isalnum` plus exactly these chars: `.-+~_=!?@#$%&|` (confirmed literal in generated lexer.cpp:206). Ends at first char outside that set (whitespace, newline, anything else). **No `/`** — subdirectory paths require the quoted form. Stored in `Lexical_analyser::file_name` (one per command, lexer.w:102).
- **`.at` defaulting**: `def_ext=".at"` is a constructor default of the interactive `BufferedInput` ctor (buffer.w:449-452), which is the one `main` uses (main.w:326-328). Appended only if open of the plain name fails and the name doesn't already end in `.at` (buffer.w:737-743).

## 2. Path resolution

- CLI option is exactly `--path=DIR` (main.w:395-401); repeatable; each dir gets `'/'` appended and lands in a **mutable user-space variable** `input_path : [string]` (main.w:414-420) — scripts can change it at runtime. Non-option CLI args = prelude filenames (main.w:402).
- Resolution loop (buffer.w:732-749): for i in 0..=path_size, prefix = `input_path[i]` for i<size, **empty string (cwd-relative) tried LAST**; each candidate tried plain, then with `.at` appended. First successful open wins; the winning full pathname is interned in `input_files_seen` (buffer.w:746).
- **No relative-to-including-file resolution** — only input_path components + cwd.
- **Prelude**: nothing is auto-loaded by the binary. The install wrapper is `exec $(INSTALLDIR)/atlas --path=$(INSTALLDIR)/atlas-scripts all.at "$@"` (top Makefile:242); `all.at` is just a list of `<basic.at`, `<combinatorics.at`, ... lines. Prelude files are read before the main loop with `push_file(name, true)`, output captured into read-only `prelude_log : [string]`; `quit`/redirect/verbosity forbidden during prelude (main.w:570-620).
- **Missing file**: `"failed to open input file 'name'."` to cerr, `push_file` returns false (buffer.w:783-788) → `include_file` calls `close_includes()` — the **entire include stack is aborted** (parsetree.w:3193-3198), session continues.
- **Cyclic include**: `push_file` scans the active `input_stack` for the same file id and **silently skips** it (no error) (buffer.w:812-815).

## 3. Include-once bookkeeping

- Identity = the **resolved pathname string** (path prefix + given name + possibly `.at`) as successfully opened, interned in `Hash_table input_files_seen` (buffer.w:746, 529). Completion tracked per id in `BitMap input_files_completed`, set only when the file is read to the end without error (`pop_file`, buffer.w:668-675).
- `<file` (skip_seen=true): skipped iff its id is in `input_files_completed` (buffer.w:811). A file aborted mid-read by an error is seen-but-not-completed → **will be re-read**. Shortcut: if the literal name as typed exactly matches a seen+completed pathname, skip without even opening (buffer.w:770-774; comment: incentive to write explicit `.at` extension in includes).
- `<<file` re-reads unconditionally (except the active-cycle check). Skipping still counts as success (buffer.w:781).
- Any error (syntax via `yyerror` main.w:196-204, runtime/type via catch blocks main.w:643-663) calls `close_includes()` → all open include files popped with "Abandoning reading of file X at line N" (buffer.w:677-685). On successful pop: `"Completely read file '...'"` printed to `*output_stream` (buffer.w:670).

## 4. Lexer inventory

**Keywords** (main.w:258-268; order must match `%token` order parser.y:65-67; token code = QUIT + offset, lexer.w:452):
`quit set let in begin end if then else elif fi and or not next do dont from downto while for od case esac rec_fun true false die break return set_type whattype showall forget`.
`true`/`false` are keywords (TRUE/FALSE). `quiet`/`verbose` are NOT keywords — ordinary identifiers guaranteed the first two id codes (main.w:283-285), recognized positionally in `set quiet|verbose` (parser.y:171-179).

**Named tokens** (parser.y:65-78): `OPERATOR OPERATOR_BECOMES '=' '*'` (carry `{id,priority}`), `INT STRING` (carry `std::string*`), `IDENT TYPE_ID` (id codes), `TOFILE ADDTOFILE FROMFILE FORCEFROMFILE`, `PRIMTYPE` (type_code), `ARROW "->"`, `BECOMES ":="`, `TLSUB "~["`, `END_OF_FILE`.

**Operators** — most collapse into generic OPERATOR with a hash-table id + priority (lexer.w:541-704):
- prio 2: `< > <= >= !=` (OPERATOR); `=` is its own token `'='` (still carries oper value)
- prio 4: `+ -`
- prio 6: `% / &` (OPERATOR); `*` is its own token `'*'`; `\` (integer div), `\%` (mod-div pair)
- prio 7: `^` (right-assoc)
- prio 8: `# ##` and operator-`~`
- Every operator above (incl. `=`, `*`, `##`, `\%`, `~`) may be immediately followed by `:=` (spaces/comments allowed between) → single token **OPERATOR_BECOMES** (`+:=` etc., lexer.w:507-516). `-` + `>` → ARROW; `:` + `=` → BECOMES, else `':'`; bare `!` → char `'!'` (const-pattern marker, parser.y:709).
- **`~` disambiguation** (lexer.w:629-651): `~[` with no space → TLSUB; `~` followed (after space/comments) by `:` `]` `,` or keyword `do|od|if|for` → plain char `'~'` (reversal marker); else OPERATOR `~` prio 8.
- Single-char passthrough tokens: `( ) [ ] , ; . : ! $ @ | ? ~ \n` and any unrecognized char as its own code (lexer.w:542-594). `$` = last value (parser.y:343); `@` = argument-less lambda / op-cast `f@type` (parser.y:225-226, 381-382). `'{' '}'` never reach the parser in atlas — they are the comment delimiters.
- Grammar's `operator` nonterminal: `OPERATOR | '=' | '*'` (parser.y:312).

**Comments**: `{ ... }`, **nested**, multi-line (set via `set_comment_delims('{','}')` main.w:288; nesting logic lexer.w:245-284). Unclosed comment at EOF → error message, char reconsidered.

**Literals**: INT = `[0-9]+`, arbitrary precision, kept as string for big_int (lexer.w:489-496). STRING = `"..."`, single line, only escape is doubled `""` → one `"` (lexer.w:297-318); broken string reported and auto-closed at newline. No char literals. Line continuation: trailing `\` at end of line joins lines at buffer level, invisible to the lexer, works mid-token (buffer.w:885-898); trailing whitespace always stripped (buffer.w:887-889).

**Identifiers**: `[A-Za-z_][A-Za-z0-9_]*` (lexer.w:395, 435-440). Classification by hash-id ranges (lexer.w:441-450): id < keyword_limit → keyword token; keyword_limit ≤ id < type_limit → **PRIMTYPE** (value = index into prim_names); else → **TYPE_ID** if `global_id_table->is_defined_type(id)` or inside `set_type [ ... ]` (state `type_defining`, where ALL identifiers scan as TYPE_ID for recursive typedefs, lexer.w:425-430, 473-477), otherwise **IDENT**. So type-identifier-ness is decided lexically by consulting the global table.

**Primitive type names** (axis-types.w:313-315 + 3573-3577, order = primitive_tag enum):
`int rat string bool vec mat ratvec LieType RootDatum WeylElt InnerClass RealForm CartanClass KGBElt Block Split KType KTypePol Param ParamPol void` — `void` is last, not a real primitive: `mk_prim_type` turns it into the empty tuple type (axis-types.w:308-310).

**Type grammar** (parser.y:792-800): `PRIMTYPE | TYPE_ID | (union_list) | (union_list_opt -> union_list_opt) | [union_list] | [*]`; tuples via `,`, unions via `|` inside parens.

**`.` field/selector syntax**: `x.f` ≡ `f(x)` — `IDENT '.' selector`, `comprim '.' selector`, `().selector` (parser.y:333-337); selector may be unit, identifier, or operator (parser.y:325-327). Field assignment `x.f := e` (parser.y:266-267), field update `x.f op:= e` (parser.y:274-276). `.` sets prevent_termination (lexer.w:552-553), and an operator right after `.` is allowed to end a command (`operator_termination`, lexer.w:528-530).

## 5. Interactive vs batch differences

- **Command termination is identical in both modes**: the grammar is per-command; a command ends at a newline seen while `nesting==0` and `prevent_termination=='\0'` (skip_space, lexer.w:215-231). Multi-line expressions in .at files must keep a bracket open, or end lines with an operator/comma/etc. (each token sets/clears `prevent_termination`, e.g. lexer.w:552-572), or use `\` continuation. The lexer then emits `'\n'` followed by a forced 0 (EOF) token (`state=ended` protocol, lexer.w:386-405) and yyparse is re-invoked per command (main.w:491-521).
- Prompt/readline only when `isatty(STDIN_FILENO)` and only at include-depth 0 (main.w:323, buffer.w:975-997). No other grammar/lexing difference.
- **End of an included file** → buffer emits `'\f'` instead of `'\n'` (buffer.w:1013); `'\f'` always breaks skip_space even inside nesting (lexer.w:221-223), becomes END_OF_FILE token (lexer.w:590), and `input: END_OF_FILE { YYABORT; }` ignores it (parser.y:137-138) — but this means **a command cannot straddle a file boundary**.
- Per completed file, `"Completely read file 'PATH'."` goes to `*output_stream` (stdout normally) (buffer.w:670); `"Starting to read from file '...'"` likewise (buffer.w:820).
- Errors anywhere close the whole include stack (main.w:202, 656); the session continues at top level. Exit status: EXIT_FAILURE if any error occurred (`clean` flag, main.w:350).
- Prelude batch mode (CLI file args): same parse loop but output redirected to `prelude_log`, `quit`/`set verbose`/redirect rejected, errors reported succinctly and reading of the file abandoned (main.w:570-620).

Key implication for the Rust port's 224/240 `<basic.at` failures: `<` must be dispatched on the *first token of a command* into filename-mode lexing (unquoted charset above, `.at` defaulting, input_path search with cwd last), with include-once keyed on resolved pathname + completed-flag, silent cycle skip, and `\f`/END_OF_FILE marking file boundaries so commands can't cross them.

---

All data collected. Here are the structured notes.

---

# axis grammar — sources/interpreter/parser.y (master 4d3e9449, 909 lines)

## 0. Parser meta
- Bison, `%define api.pure`, `%locations`, `%parse-param {expr_p* parsed_expr} {int* verbosity}`, `%define parse.error verbose` (parser.y:59-63).
- Tokens (parser.y:65-78): keywords `QUIT SET LET IN BEGIN END IF THEN ELSE ELIF FI AND OR NOT NEXT DO DONT FROM DOWNTO WHILE FOR OD CASE ESAC REC_FUN TRUE FALSE DIE BREAK RETURN SET_TYPE WHATTYPE SHOWALL FORGET`; `<oper> OPERATOR OPERATOR_BECOMES '=' '*'` (note: `=` and `*` are distinct tokens but carry oper values); `<str> INT STRING`; `<id_code> IDENT TYPE_ID`; `TOFILE ADDTOFILE FROMFILE FORCEFROMFILE`; `<type_code> PRIMTYPE`; `ARROW "->"`; `BECOMES ":="`; `TLSUB "~["`; `END_OF_FILE`.
- **The ONLY precedence declaration is `%left OPERATOR` (parser.y:104).** All real operator precedence is NOT in the grammar: the lexer attaches `{id, priority}` to each operator token and parse actions (`start_formula`/`start_unary_formula`/`extend_formula`/`end_formula`) restructure the tree (parser.y:302-309; parsetree.w:1110-1127). Associativity convention: **even priority = left-assoc, odd = right-assoc** (parsetree.w:1123-1127).
- Operator priority table (assigned in lexer.w): `< > <= >= = !=` prio 2 (lexer.w:564-582); `+ -` prio 4 (lexer.w:658-667); `* % / & \ \%` prio 6 (lexer.w:671-688); `^` prio 7 right-assoc (lexer.w:692-693); `# ##` prio 8 (lexer.w:696-701); `~` as operator prio 8 (lexer.w:647-648). Unary use of any operator: `operand: operator operand` → `make_unary_call` (parser.y:314).
- Lexer specials: `~[` must be a single token TLSUB, no space (lexer.w:632-641); bare `~` is the special reversal marker only when followed by `: ] , do od if for`, else it is an operator (lexer.w:636-650); any operator immediately followed by `:=` lexes as OPERATOR_BECOMES (`becomes_follows()`); at command start `<` `<<` `>` `>>` lex as FROMFILE/FORCEFROMFILE/TOFILE/ADDTOFILE with a filename (lexer.w:553-560).
- Keyword spellings, in %token order: main.w:258-268 (`"quit","set","let","in","begin","end","if","then","else","elif","fi","and","or","not","next","do","dont","from","downto","while","for","od","case","esac","rec_fun","true","false","die","break","return","set_type","whattype","showall","forget"`).
- PRIMTYPE names (axis-types.w:313-315 + 3573-3577): `int rat string bool vec mat ratvec LieType RootDatum WeylElt InnerClass RealForm CartanClass KGBElt Block Split KType KTypePol Param ParamPol void` — `"void"` is special: makes empty tuple type, not a primitive (axis-types.w:308-311).

## 1. Top level — `input` (parser.y:136-218)
```
input : '\n'                            -- null input, YYABORT
      | END_OF_FILE                     -- YYABORT
      | expr '\n'                       -- *parsed_expr = expr  (the ONLY case that evaluates)
      | SET declarations '\n'           -- global_set_identifiers        (set a=e, b=f, ...)
      | FORGET IDENT '\n' | FORGET TYPE_ID '\n'          -- global_forget_identifier
      | SET operator '(' id_specs ')' '=' expr '\n'      -- define operator as lambda (overload, flag 2)
      | SET operator '=' expr '\n'                       -- overload-define operator
      | FORGET IDENT '@' type '\n' | FORGET operator '@' type '\n'  -- forget one overload
      | IDENT ':' expr '\n'             -- global_set_identifier(id,e,0)  (id : e definition)
      | IDENT ':' type '\n'             -- global_declare_identifier (forward declaration)
      | SET_TYPE IDENT '=' type_spec '\n' | SET_TYPE TYPE_ID '=' type_spec '\n'  -- type_define_identifier
      | SET_TYPE '[' type_equations ']' '\n'             -- process_type_definitions (recursive group)
      | QUIT '\n'                       -- *verbosity = -1
      | SET IDENT '\n'                  -- 'set quiet'(0) / 'set verbose'(1) by identifier code (parser.y:171-179)
      | TOFILE expr '\n'                -- eval with output to file (*verbosity=2)
      | ADDTOFILE expr '\n'             -- append output (*verbosity=3)
      | FROMFILE '\n'                   -- include_file(1)   ('< file')
      | FORCEFROMFILE '\n'              -- include_file(0)   ('<< file')
      | WHATTYPE expr '\n' | WHATTYPE TYPE_ID '\n' | WHATTYPE TYPE_ID '?' '\n'
      | WHATTYPE id_op '?' '\n'                          -- show_overloads
      | TOFILE WHATTYPE id_op '?' '\n' | ADDTOFILE WHATTYPE id_op '?' '\n'
      | SHOWALL '\n' | TOFILE SHOWALL '\n' | ADDTOFILE SHOWALL '\n' ;
id_op : IDENT | operator ;              -- (parser.y:220-222)
```
Every non-expr alternative ends with YYABORT (nothing to evaluate). Note `'\n'` terminates every input unit.

## 2. Expression hierarchy (top-down: expr > tertiary > or > and > not > secondary > formula/primary)
```
expr    : LET lettail
        | '@' ':' expr                       -- no-arg lambda
        | '@' cast                           -- no-arg lambda, body is 'type:expr'
        | '(' id_specs ')' ':' expr          -- lambda
        | '(' id_specs ')' cast              -- lambda with cast body
        | REC_FUN IDENT '(' id_specs_opt ')' type ':' expr   -- recursive lambda (result type mandatory)
        | RETURN expr
        | cast
        | tertiary ';' expr                  -- make_sequence(.,.,0)  (discard first)
        | tertiary NEXT expr                 -- make_sequence(.,.,1)  (keep first, 'next')
        | tertiary ;                                            (parser.y:224-238)
cast    : type ':' expr ;                    -- make_cast              (parser.y:240)
lettail : declarations IN expr | declarations THEN lettail ;  -- 'let a=1 then b=a+1 in e' (242-244)
declarations : declarations ',' declaration | declaration ;   (246-248)
declaration  : pattern '=' expr
        | IDENT '(' id_specs_opt ')' '=' expr                 -- let-level function shorthand
        | REC_FUN IDENT '(' id_specs_opt ')' '=' type ':' expr ;   (250-261)

tertiary: IDENT BECOMES tertiary                 -- x := e
        | SET pattern BECOMES tertiary           -- set (a,b) := e   (multi-assignment)
        | assignable_subsn BECOMES tertiary      -- a[i] := e
        | IDENT '.' IDENT BECOMES tertiary       -- r.f := e   (field assignment)
        | IDENT OPERATOR_BECOMES tertiary        -- x +:= e  ==> x := x + e (desugared, parser.y:268-271)
        | assignable_subsn OPERATOR_BECOMES tertiary    -- a[i] +:= e (make_comp_upd_ass)
        | IDENT '.' ident_expr OPERATOR_BECOMES tertiary -- r.f +:= e (make_field_upd_ass)
        | or_expr ;                                            (263-278)
or_expr : or_expr OR and_expr | and_expr ;   -- desugar: if a then true else b (280-284)
and_expr: and_expr AND not_expr | not_expr ; -- desugar: if a then b else false (286-290)
not_expr: NOT not_expr | secondary ;         -- make_negation (292-294)
secondary: formula | '(' ')' | primary ;     -- '()' = empty tuple display, excluded from call/subscript head (296-300)

formula : formula_start operand ;                              (302-303)
formula_start : operator                     -- unary operator start
        | comprim operator | ident_expr operator
        | formula_start operand operator ;   -- priority resolution in actions (304-309)
operator: OPERATOR | '=' | '*' ;             -- '=' and '*' double as operators (312)
operand : operator operand                   -- unary call (chain of prefix ops)
        | primary ;                                            (314-316)
tilde_opt : '~' { 1 } | /*empty*/ { 0 } ;    -- reversal marker (319-321)

primary : comprim | ident_expr ;                               (323)
ident_expr : IDENT ;                         -- applied identifier (324)
selector: unit | ident_expr | operator ;     -- what may follow '.' (325-327)

comprim : subscription | slice
        | primary '(' commalist_opt ')'      -- call f(a,b,...)
        | IDENT '.' selector                 -- x.f  ==> f(x)  (postfix application)
        | comprim '.' selector               -- chained: e.f  ==> f(e)
        | '(' ')' '.' selector               -- ().f
        | unit ;                                               (330-338)
```
### `unit` — atoms and structured expressions (parser.y:339-386)
```
unit    : INT | TRUE | FALSE | STRING
        | '$'                                -- make_dollar (last value)
        | IF iftail
        | IF expr ELSE expr THEN expr FI     -- inverted if (else-branch first!)
        | CASE expr IN commalist ESAC                             -- int case
        | CASE expr IN commalist ELSE expr ESAC                   -- + out-of-range branch
        | CASE expr ELSE expr IN commalist ESAC                   -- (else first)
        | CASE expr IN commalist THEN expr ELSE expr ESAC         -- THEN = negative branch, ELSE = overflow
        | CASE expr THEN expr IN commalist ELSE expr ESAC
        | CASE expr THEN expr ELSE expr IN commalist ESAC         -- all 6 orderings of IN/THEN/ELSE
        | CASE expr IN barlist ESAC          -- union case (make_union_case_node)
        | CASE expr '|' caselist ESAC        -- tagged discrimination (make_discrimination_node)
        | WHILE do_expr tilde_opt OD         -- while loop (body inside do_expr via DO)
        | iffor_loop
        | '(' expr ')' | BEGIN expr END      -- grouping (begin/end == parens)
        | '[' commalist_opt ']'              -- list display
        | '[' commabarlist ']'               -- matrix display [r1|r2|...]: desugars to
                                             --   transpose(mat: [row-lists]) (parser.y:370-376)
        | '(' commalist ',' expr ')'         -- tuple display (>= 2 components)
        | operator '@' type | IDENT '@' type -- operator/identifier cast (overload selection)
        | DIE | BREAK | BREAK INT ;
```
### Lists (parser.y:388-410)
```
commalist_opt : /*empty*/ | commalist ;
commalist : expr | commalist ',' expr ;
barlist   : expr '|' expr | barlist '|' expr ;          -- >= 2 items
commabarlist : commalist '|' commalist | commabarlist '|' commalist ;  -- matrix rows
```
### if / case tails (parser.y:412-436)
```
iftail  : expr THEN expr ELSE expr FI | expr THEN expr ELIF iftail
        | expr THEN expr FI ;                -- missing else = () (415)
caselist: IDENT closed_pattern ':' expr      -- tag(pat): e
        | pattern '.' IDENT ':' expr         -- pat.tag: e   (alternate order)
        | IDENT ':' expr                     -- tag: e  (no pattern)
        | ELSE expr                          -- default (tag -1)
        | caselist '|' <each of the above 4> ;
```

## 3. do_expr — bodies of while and guarded loops (parser.y:438-504)
Parallel universe of `expr` where `DO` splits guard from body: `make_sequence(...,2)` = "do" kind.
```
do_expr : LET do_lettail
        | tertiary ';' do_expr               -- sequence, kind 0
        | tertiary DO expr                   -- guard DO body: sequence kind 2
        | DO expr                            -- no guard: 'true DO body'
        | DONT                               -- 'false DO die'
        | IF do_iftail
        | IF expr ELSE do_expr THEN do_expr FI
        | CASE ... (same 8 int/union/discrimination shapes as unit, with do_commalist/do_barlist/do_caselist)
        | '(' do_expr ')' ;
do_lettail : declarations IN do_expr | declarations THEN do_lettail ;
do_iftail  : expr THEN do_expr ELSE do_expr FI | expr THEN do_expr ELIF do_iftail ;
do_commalist / do_barlist / do_caselist : same shapes as commalist/barlist/caselist with do_expr bodies (476-504)
```

## 4. Loops (parser.y:506-575)
```
iffor_loop : if_loop | for_loop ;
if_loop : IF expr DO expr FI     -- desugar: cond ? [body] : []  (list of 0/1 elts, parser.y:509-515)
        | IF expr iffor_loop FI  -- cond ? loop : []
for_loop:
  FOR pattern_opt IN expr ~? DO expr ~? OD               -- for x in L do ... od; flags = in_rev + 2*body_rev
| FOR pattern_opt IN expr ~? iffor_loop ~? OD            -- body is nested loop => wrap in unary "## " (flatten)
| FOR pattern_opt '@' IDENT IN expr ~? DO expr ~? OD     -- for x@i in L: index variable
| FOR pattern_opt '@' IDENT IN expr ~? iffor_loop ~? OD
| FOR IDENT ':' expr ~? DO expr ~? OD                    -- counted: for i:n  (make_cfor_node, from 0)
| FOR IDENT ':' expr ~? iffor_loop ~? OD
| FOR IDENT ':' expr FROM expr ~? DO expr ~? OD          -- for i:n from k
| FOR IDENT ':' expr FROM expr ~? iffor_loop ~? OD
| FOR ':' expr DO expr ~? OD                             -- anonymous repeat n times (id -1, flag +4)
| FOR ':' expr iffor_loop ~? OD
| FOR IDENT ':' expr DOWNTO expr DO expr OD              -- for i:n downto k (flag 1, no ~ allowed)
| FOR IDENT ':' expr DOWNTO expr iffor_loop OD ;
```
(`~?` = tilde_opt.) IN-loops build a 2-component pattern (value, index-or-empty) (parser.y:524-526). Loop-in-loop-body desugars to `##`(flatten) of list-of-lists (parser.y:532 etc.).

## 5. Subscription and slices (parser.y:578-706)
```
assignable_subsn : IDENT '[' expr ']' | IDENT TLSUB expr ']'          -- x[i], x~[i] (reversed)
        | IDENT '[' expr ',' expr ']' | IDENT TLSUB expr ',' expr ']' -- m[i,j] (tuple subscript)
subscription : assignable_subsn
        | comprim '[' expr ']' | comprim TLSUB expr ']'
        | comprim '[' expr ',' expr ']' | comprim TLSUB expr ',' expr ']' ;
expr_opt : expr | /*empty*/ ;                                          (622)
slice   : (IDENT|comprim) ('['|TLSUB) expr_opt ~? ':' expr_opt ~? ']'  -- 1-D slice a[i~:j~], a~[...]
        | (IDENT|comprim) '[' expr_opt ~? ':' expr_opt ~? ',' expr_opt ~? ':' expr_opt ~? ']'
          -- 2-D matrix slice: desugars to call of builtin "matrix slicer" with flag word +
          -- (mat cast of subject, lo_r, hi_r, lo_c, hi_c) tuple (parser.y:658-705)
```
Flag encoding for 1-D slices: bit0 = subject reversed (TLSUB), bit1 = lower reversed, bit2 = upper reversed; omitted lower defaults to 0/flag 0x0, omitted upper defaults flag 0x4 (parser.y:624-656).

## 6. Patterns (parser.y:708-729)
kind bitmask (raw_id_pat): 0x1 = has name, 0x2 = has sublist, 0x4 = const ("!"), 0x0 = wildcard/empty.
```
pattern : IDENT                              -- 0x1
        | '!' IDENT                          -- 0x5, declared const/transient
        | closed_pattern
        | '(' pat_list ')' ':' IDENT         -- 0x3: tuple pattern AND whole-value name
        | '(' pat_list ')' ':' '!' IDENT ;   -- 0x7: same, const
closed_pattern : '(' pattern ')' | '(' pat_list ')' /*0x2*/ | '(' ')' /*0x2, discard*/ ;
pattern_opt : /*empty (0x0)*/ | pattern ;
pat_list : pattern_opt ',' pattern_opt | pat_list ',' pattern_opt ;    -- >= 2 components, holes allowed
```

## 7. Parameter specs (lambda params) (parser.y:731-790)
```
id_spec : type pattern                       -- e.g. 'int n' or '(int,int)(a,b)'
        | '(' id_specs ')'                   -- nested: tuple type built from component specs
        | type '.' ;                         -- anonymous (discarded) parameter
id_specs : id_spec | id_specs ',' id_spec ;  -- builds parallel (type list, pattern list)
id_specs_opt : id_specs | /*empty*/ ;
type_spec : type                             -- for SET_TYPE rhs
        | '(' struct_specs ')'               -- struct: tuple type with field names
        | '(' union_specs ')' ;              -- union with variant names
struct_specs : type_field ',' type_field | struct_specs ',' type_field ;
union_specs  : type_field '|' type_field | union_specs '|' type_field ;
type_field : type IDENT | type '.' ;         -- named or anonymous field (788-790)
```

## 8. Types (parser.y:792-818)
```
type : PRIMTYPE                              -- one of the 20 prim_names
     | TYPE_ID                               -- previously defined type abbreviation
     | '(' union_list ')'                    -- parenthesized/tuple/union type
     | '(' union_list_opt "->" union_list_opt ')'   -- function type; either side may be empty (= void)
     | '[' union_list ']'                    -- row-of type
     | '[' '*' ']' ;                         -- row of unknown component type
union_list_opt : /*empty => singleton void*/ | union_list ;
union_list : type | types                    -- one variant (types => it's a tuple)
     | union_list_opt '|'                    -- trailing empty variant (void)
     | union_list_opt '|' type | union_list_opt '|' types ;
types : type ',' type | types ',' type ;     -- tuple of >= 2 (815-818)
```
So: `(int,int)` tuple, `(int|string)` union, `(int->bool)` function, `[vec]` row, `(->)` void→void. Void = empty tuple.

## 9. Recursive type definitions — `set_type [ ... ]` (parser.y:820-906)
```
type_equations : type_equation | type_equations ',' type_equation ;
type_equation  : TYPE_ID '=' typedef_type | '.' '=' typedef_type ;   -- '.' = anonymous (id -1)
typedef_type : '[' typedef_list ']' | '(' typedef_composite ')'
     | '(' typedef_list_opt "->" typedef_list_opt ')'
     | '(' typedef_struct_specs ')' | '(' typedef_union_specs ')'
     | PRIMTYPE ;
typedef_list_opt : /*empty*/ | typedef_list ;
typedef_list : typedef_unit | typedef_composite ;
typedef_composite : typedef_units | typedef_list_opt '|' | typedef_list_opt '|' typedef_unit
     | typedef_list_opt '|' typedef_units ;
typedef_units : typedef_unit ',' typedef_unit | typedef_units ',' typedef_unit ;
typedef_unit : TYPE_ID | typedef_type | '(' typedef_unit ')' ;
typedef_struct_specs / typedef_union_specs : like struct_specs/union_specs over typedef_type_field ;
typedef_type_field : typedef_unit TYPE_ID | typedef_unit '.' ;   -- field names are TYPE_ID here (903-906)
```
Mirror of the `type` grammar but with TYPE_ID allowed self-referentially (enables recursion within the bracket group).

## Key desugarings done IN the parser (Rust port must replicate or absorb)
1. `a OR b` → `if a then true else b`; `a AND b` → `if a then b else false` (parser.y:280-288).
2. `x op:= e` → `x := op(x,e)`; component/field variants use dedicated nodes (263-276).
3. `x.f`, `e.f(...)` → application `f(x)` (333-337).
4. `[r1|r2]` matrix display → `transpose(mat:[[r1],[r2]])` via builtin id `"transpose "` (370-376).
5. `if c do e fi` (loop guard form) → conditional yielding 1- or 0-element list (509-520).
6. for-loop with nested-loop body → `##`(loop) flatten via builtin id `"## "` (532,543,552,...).
7. 2-D slices → call of builtin `"matrix slicer"` with packed flag int (658-705).
8. Sequence kinds: `;`=0, `NEXT`=1, `DO`=2 in `make_sequence` (235-236, 439-443).
9. Inverted forms exist everywhere: `IF c ELSE e THEN t FI`, `CASE e ELSE d IN l ESAC`, etc. — all 6 permutations for int-case.
10. `quiet`/`verbose` are NOT keywords — they are the two lowest identifier codes, recognized inside `SET IDENT` (parser.y:171-179, main.w:272-276).

Constructor inventory referenced by actions (all in parsetree.w): make_lambda_node, make_rec_lambda_node, make_return, make_cast, make_op_cast, make_let_expr_node, make_let_node/append_let_node, make_assignment, make_multi_assignment, make_comp_ass, make_comp_upd_ass, make_field_ass, make_field_upd_ass, make_conditional_node, make_int_case_node (3 arities), make_union_case_node, make_discrimination_node, make_case_node/append_case_node, make_while_node, make_for_node, make_cfor_node, make_application_node, make_unary_call, make_binary_call, make_subscription_node, make_slice_node, make_sequence, make_negation, wrap_tuple_display, wrap_list_display, make_exprlist_node/reverse_expr_list, make_int/bool/string_denotation, make_dollar, make_die, make_break, make_applied_identifier, start/extend/end_formula, start_unary_formula, pattern/type list builders (make_pattern_node, make_type_singleton/list, make_prim/tuple/union/function/row_type, make_typedef_singleton, append_typedef_node).

---

Research complete. Below are the structured notes.

# ORACLE NOTES: definition semantics & session output (atlasofliegroups @ 4d3e9449, sources/interpreter/)

## 1. `set` — all forms (parser.y grammar + global.w semantics)

**Grammar forms (parser.y:136-218, 246-261):**
| Form | Production | Handler | `overload` arg |
|---|---|---|---|
| `set <declarations>` | parser.y:140 | `global_set_identifiers` → `do_global_set(...,1,...)` (global.w:777-780) | 1 (allow both) |
| declaration: `pattern = expr` | parser.y:250 | (via zip_decls, same as `let`) | — |
| declaration sugar: `f (int n,...) = expr` | parser.y:251-254 — desugars to `f = lambda` via `make_lambda_node` | — | — |
| declaration: `rec_fun f (specs) = type: expr` | parser.y:255-260 (`make_rec_lambda_node`) | — | — |
| `set <op> (id_specs) = expr` | parser.y:143-150 (wraps lambda) | `global_set_identifier(id,...,2,...)` | 2 (require overload) |
| `set <op> = expr` | parser.y:151-154 | same | 2 |
| `IDENT : expr` (no `set` keyword!) | parser.y:159-161 | `global_set_identifier(id,e,0,...)` | 0 (never overload) |
| `set IDENT` alone | parser.y:171-179 = option toggle; only `quiet`(→verbosity 0)/`verbose`(→1); else stderr `'name' is not something one can set` | | |

`operator : OPERATOR | '=' | '*'` (parser.y:312). Patterns: `IDENT`, `!IDENT` (const), `(p1,p2,...)` tuple, `(list):IDENT` (whole+components), `()` discard (parser.y:708-729). Multiple `set` declarations separated by commas: parser.y:246-248.

**do_global_set semantics (global.w:911-948), 3 phases** — this ordering is observable:
- **phase 0** typecheck: `analyse_types` (global.w:651-684; errors→stderr, then throws), then `pattern_type(pat).specialise(t)` must succeed else `Type T of right hand side does not match required pattern P` (global.w:1048-1055); `overload==2` with non-function type → `Cannot set operator 'x' to a value of non-function type T` (global.w:958-965). `definition_group::add` rejects the same identifier twice in one command (`Multiple occurrences of 'x' cannot be defined in same definition`, global.w:856-865) and *pre-checks* overload conflicts via `locate_overload` (global.w:867-871).
- **phase 1** `e->eval()` — the whole RHS evaluated once (global.w:926-927). **Any `prints`/output from RHS appears BEFORE the definition report lines.**
- **phase 2** values split by `thread_components` in pattern (depth-first, left-to-right) order, one report line per identifier (global.w:929-945). Routing per identifier: id table iff `overload==0 or type is not a function type` (global.w:939); function values with `overload>=1` go to overload table.

**Which table:** ordinary `Id_table` = one binding per name, silently replaced on re-add (global.w:196-205). `overload_table` = per-name vector of (arg-type, value) sorted most-specific-first (global.w:389-410). The two tables are independent — no cross-check or cross-removal: `set f=(int n):n` then `set f=3` leaves both bindings live.

**Overload vs replace rule** (`overload_table::add`, global.w:515-557 + `locate_overload` global.w:466-492 + `is_close` axis-types.w:3191-3201, 3257-3286):
- `is_close(new_arg, existing_arg)` returns 3 bits: `0x0` unrelated → coexist; `0x6` (existing converts to new) → new inserted after; `0x5` (new converts to existing) → inserted before; `0x7` with **equal** arg types → **replace in place** (slot index preserved, global.w:552-554); `0x7` non-equal (mutually convertible, e.g. `vec` vs `[int]`) or `0x4` (close but no direction) → throw:
  `` Cannot overload `f':\nalready overloaded type 'T1' is too close to new argument type 'T2',\nwhich would make overloading ambiguous for certain arguments. Simultaneous\noverloading for these types is not possible, forget the other one first.`` (global.w:501-510)
- `is_close` core: equal types 0x7; `void`/`*` close to nothing; primitives compared via `coerce` in both directions (|0x4); rows compare componentwise; tuples AND componentwise with equal length; other kinds close only if equal (axis-types.w:3257-3286). Overloading never specialises `*` (axis-types.w:3231-3234).

## 2. `set_type`

**Simple form** `set_type Id = type_spec` (parser.y:163-166; both fresh `IDENT` and existing `TYPE_ID` accepted → redefinition legal) → `type_define_identifier` (global.w:1289-1323). `type_spec` allows field names: `(int first, rat second)` tuple / `(int a | string b)` union (parser.y:753-790). Checks: field name = type name → error (global.w:1375-1381); name in use as ordinary variable/function → `Cannot define 'X' as a type; it is in use as global variable|function` (global.w:1510-1520); redefinition of an existing *type* id allowed, old pro/injectors cleaned out first (`clean_out_type_identifier`, global.w:1176-1234 — removes field functions from overload table only if still the exact projector/injector values). Stored via `Id_table::add_type_def` — entry with null value slot marks "defined type" (global.w:216-229). Field names get projector (`(WholeType->component)`) / injector (`(component->WholeType)`) functions in the overload table (global.w:1334-1359, 1398-1408).

**Bracketed (recursive) form** `set_type [ Id1 = rhs1, Id2 = rhs2, ... ]` (parser.y:167-168, 820-850) → `process_type_definitions` (global.w:1424-1459). Scanner treats *all* identifiers as type ids inside the command; RHS may not be a bare type identifier (global.w:1522-1533). Identifier→type-number resolution via `translate` table; errors: `Repeated definition of 'X'` (global.w:1485-1488), `Used 'X' as defined type AND as field name` (global.w:1492-1497), `Type identifier 'X' does not refer to any type` (global.w:1582-1586). Only this form records the binding in `type_expr::type_map`, so the *name* prints in later type output (axis-types.w:1668-1673); rollback of the type table on error (global.w:1452).

**Later mentions:** the `type` production for `TYPE_ID` copies the stored type out of `global_id_table` *at parse time* (parser.y:792-794) — simple typedefs are structural aliases, expanded immediately.

## 3. `forget`

- `forget name` (IDENT or TYPE_ID; parser.y:141-142) → `global_forget_identifier` (global.w:1241-1248): if defined type, `clean_out_type_identifier` first; removes from `Id_table` only (overloads survive).
- `forget name@type` / `forget op@type` (parser.y:155-158) → `global_forget_overload(id, t)` (global.w:1254-1261); `type` after `@` is the **argument type only**; removal requires exact `arg_type` equality (global.w:565-576).

## 4. `:` declarations at top level

- `IDENT : type` → `global_declare_identifier` (parser.y:162; global.w:1159-1167): enters id with type and **null value** (must be assigned via `x := ...` before use).
- `IDENT : expr` → define with value, `overload=0` — always the ordinary id table even for function values (parser.y:159-161). There is **no** bare `ident = value` top-level form; `=` at top level is only inside `set`/`set_type`, and standalone assignment is the expression `x := e`.

## 5. OUTPUT SURFACE (byte-exact; all to `*output_stream` = stdout unless noted)

**Indentation:** every definition-report line is prefixed by `2*include_depth` spaces (`<< std::setw(2*input_level) << ""`, global.w:1039-1042; depth = open include stack size, buffer.w:1062). The `Starting.../Completely read...` lines are NOT indented.

| Event | Exact text (then `std::endl`) | Cite |
|---|---|---|
| set, non-function (or overload=0) | `{indent}Variable x: int` — `Constant` instead of `Variable` for `!x`; if name existed in id table, append `` (overriding previous instance, which had type OLDTYPE)`` with `` (constant)`` inserted before the `)` if old was const | global.w:973-984 |
| set, first overload of name | `{indent}Defined f: (int->int)` — full function type with parens; multi-arg prints `(int,int->int)` | global.w:1013-1021; func printing axis-types.w:1620-1635 |
| set, same arg type again | `{indent}Redefined f: (int->int)` | global.w:1013-1014 |
| set, new overload | `{indent}Added definition [n] of f: (rat->rat)` — n = new total variant count (2nd overload prints `[2]`) | global.w:1012-1018 |
| `IDENT : type` | `{indent}Declaring identifier 'x': int` | global.w:1162-1164 |
| `forget x` | `Identifier 'x' forgotten` / `Identifier 'x' not known` (no indent emitted) | global.w:1245-1247 |
| `forget f@T` | `Definition of 'f@T' forgotten` / `...' not known` (T = argument type) | global.w:1258-1260 |
| `set_type` simple | `{indent}Type name 'P' defined as (int,rat)` (`redefined as` on redefinition), then if fields: `{indent}  with projectors: a, b.` (`injectors` for union) | global.w:1388-1391, 1399-1408 |
| `set_type [...]` | per equation: `{indent}Type name 'X' defined as <expanded rhs>` (via `.untabled()`) or `Anonymous type T`; then fields line as above (holes in field list print as empty item between commas) | global.w:1641-1654, 1710-1731 |
| top-level expr, non-void type | `Value: <value>` | main.w:543 |
| top-level expr, void type | *nothing* (void_eval, print suppressed) | main.w:537-538 |
| include push | `Starting to read from file 'PATH'.` — PATH = resolved path as opened (search-path prefix + name + maybe auto `.at` ext) | buffer.w:820-821, 726-750 |
| include pop | `Completely read file 'PATH'.` | buffer.w:670-671 |
| re-include of completely-read file via `<` | **silent, nothing printed** (skip); `<<` forces re-read | buffer.w:770-827 |
| include open failure | stderr: `failed to open input file 'name'.`; aborts all nested includes | buffer.w:786, parsetree.w:3193-3198 |
| include aborted by error | stderr: `Abandoning reading of file 'F' at line N` per open file | buffer.w:679-680 |
| `whattype expr` | `Type: T` | global.w:1753 |
| `whattype TypeId` | `Defined type: <expansion>`; with fields: `( int a , rat . )` style — `( T1 name1 , T2 name2 )`, `.` for anonymous, `|` sep for unions | global.w:1768-1781 |
| `whattype f?` | `Overloaded instances of 'f'` (or `No overloads for 'f'`) then per variant `  arg->res` — **two-space indent, NO outer parens** | global.w:1790-1799 |
| `showall` | `Overloaded operators and functions:\n` + overload table (`name: (arg->res): value` per line) + `Global variables:\n` + id table (`name: type: value`, value `*` if unset, `name = type` for typedefs) | global.w:1806-1811, 584-588, 300-315 |

**Error surface (stderr):**
- set failure: `Error in 'set' command at FILE:LINE:COL-COL:\n<msg>\n  Command 'set <pat>' not executed|interrupted|failed` + (phase<2) `, nothing defined` (`overloaded` if overload==2) + `.\n`; sets `clean=false`, closes includes (global.w:1117-1130). `source_location` prints `at FILE:l:c-c` (parsetree.w:173-180).
- typedef failure: `Error in type definition at ...:\n<msg>\n  Type definition aborted` (global.w:1316-1322); `Error in 'set_type' command at ...` for bracketed form (global.w:1451-1458).
- type errors from `analyse_types`: `Error during analysis of expression <loc>` / `...has wrong type: found A while B was needed.` variants (global.w:651-684).
- main-loop runtime: `Runtime error:\n  ` (or `Internal error: `) + msg + `\nEvaluation aborted.\n` (main.w:643-663).

**Exit status:** `clean` flag (global.w:1063-1070) set false on any error → drives exit code; this is why 224/240 failing at `<basic.at` matters for differential comparison.

## 6. Evaluation order / interleaving

- Per `set`: typecheck fully → evaluate RHS **once** (its side-effect output interleaves here) → then per-identifier report lines in pattern order (global.w:911-948). A phase-2 overload conflict can fire *after* earlier identifiers of the same command were already added+reported (`add_overload` prints only after successful table `add`, global.w:992-1022).
- Included files execute command-by-command; each command's reports appear immediately (indented), nested `Starting.../Completely read...` bracket them; `pop_file` happens when the file's last line is consumed.
- Prelude files (main.w:570-620): all normal output captured to `prelude_log` string, NOT stdout; `Value:` lines there use `'\n'` not endl (main.w:597); errors still go to stderr.
- Interactive banner/prompt `atlas> ` only when isatty (main.w:323-344); piped input has no prompt, no echo.
