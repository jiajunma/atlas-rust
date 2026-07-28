# KL-chain oracle trace (upstream master 4d3e9449)

Raw trace notes for task #14 (blocks + KL). Three independent
read-only traces of the upstream C++; design docs will cite these.

---

# Block construction — oracle notes (atlasofliegroups @ master 4d3e9449)

All paths relative to `/Users/hoxide/mycodes/atlasofliegroups/sources`.

## 1. What a block element IS

- Element = pair `(x,y)`: `Block_base::EltInfo { KGBElt x,y; DescentStatus descent; unsigned short length; }` — gkmod/blocks.h:62-80. `x`,`y` are *indices into KGB sets "which might no longer exist"* (blocks.h:64).
- For classic `Block`: `x` ∈ KGB(G_R), `y` ∈ KGB(dual real form of the dual inner class) — a genuine second KGB, not a TorusElement. Header comment: block parametrized by pairs (x,y) with θ_y = −θ_x^t (blocks.h:242-264, esp. 259-263).
- Defining data: a real form and a dual real form in one inner class. `Block::build(InnerClass&, RealFormNbr rf, RealFormNbr drf)` constructs `RealReductiveGroup G_R(G,rf)`, `InnerClass dG(G,DualTag())`, `RealReductiveGroup dG_R(dG,drf)`, then `KGB kgb(G_R, common_Cartans(G_R,dG_R)); KGB dual_kgb(dG_R, common_Cartans(dG_R,G_R)); return Block(kgb,dual_kgb);` — blocks.cpp:610-619. Second overload `build(G_R,dG_R)` uses stored full KGBs — blocks.cpp:622-626. `common_Cartans = GR.Cartan_set() & innerClass().dual_Cartan_set(dGR.realForm())` — blocks.cpp:1727-1728. KGB restricted-Cartans ctor: kgb.h:313.
- Interpreter chain (Fokko `block` command): block mode is a descendant of real mode (interface/commands.cpp:108); mode entry prompts for dual real form via `interactive::get_dual_real_form` (interface/blockmode.cpp:203-222); `block_f()` → `currentBlock().print_to(file,false)` (blockmode.cpp:326-330) with `currentBlock()` lazily doing `new Block(Block::build(currentRealGroup(),currentDualRealGroup()))` (blockmode.cpp:164-171). Command registered "prints all the representations in a block" (blockmode.cpp:110-111).
- `common_block` (non-integral parameter blocks): element representative is `StandardReprMod { KGBElt x_part; RatWeight gamlam /* real_unique(γ−λ) */ }` — gkmod/repr.h:122-151. Here `y` in `EltInfo` is just a locally invented counter numbering distinct `gamma_lambda` values per involution (blocks.cpp:834-841, 930-943, 1010-1021, 1137-1145); no dual KGB exists.

## 2. Generation algorithm

### Classic `Block` (fibred product, blocks.cpp:526-606)
- No orbit search: it is a *product over matched twisted-involution packets*. For each involution `w` of kgb (i in 0..nr_involutions), compute `dual_w = dual_involution(w,tW,dual_tW)` (blocks.cpp:535-543), then Cartesian product `tauPacket(w) × dual_tauPacket(dual_w)`: `info.push_back(EltInfo(x,y,descents(x,y,kgb,dual_kgb),kgb.length(x)))` — blocks.cpp:548-558.
- Length: `length(z) = kgb.length(x(z))` — blocks.cpp:557 (not re-normalized).
- Cross action: acts on BOTH sides simultaneously — `cross_image = element(kgb.cross(s,x(z)), dual_kgb.cross(s,y(z)))` — blocks.cpp:570-571.
- Cayley: x-side forward Cayley, y-side INVERSE Cayley (the asymmetry): ImaginaryTypeII: `element(kgb.cayley(s,x), dual_kgb.inverseCayley(s,y).second)` stored as `Cayley_image.second`, with back-link `data[s][z1].Cayley_image.first = z` (blocks.cpp:575-581), FALL THROUGH to TypeI: `element(kgb.cayley(s,x), dual_kgb.inverseCayley(s,y).first)` → `Cayley_image.first`; reverse link via `first_free_slot` (blocks.cpp:583-590).
- One shared field per (s,z): `block_fields { BlockElt cross_image; BlockEltPair Cayley_image; }` (blocks.h:83-89). Direction disambiguated by descent status: `cayley()` returns the pair only if NOT weak descent; `inverseCayley()` only if weak descent; else (Undef,Undef) — blocks.h:143-155.
- Lookup `element(x,y)`: via `d_first_z_of_x` table + consecutive-y property — blocks.cpp:242-248, table built at 630-643.
- Extra per-element `Block` fields: `d_Cartan[z]=kgb.Cartan_class(x)`, `d_involution[z]=kgb.involution(x)` (blocks.cpp:596-603); `d_involutionSupport` via first strict descent recursion (blocks.cpp:646-683).
- 5-configuration local-structure table (Complex / ImagTypeI↔RealTypeI / ImagTypeII↔RealTypeII / ImaginaryCompact / RealNonparity) documented in the big comment blocks.cpp:34-121.

### `common_block` full constructor (seed + closure, blocks.cpp:733-1081)
- Seed = a `StandardReprMod` srm. Step 2: raise z to most-split fiber by repeated complex-ascent cross / `up_Cayley` on imaginary-noncompact s until none (blocks.cpp:761-784); sets `highest_x`.
- Step 3: generate initial real-fiber orbit of y's at fixed `highest_x` by cross actions along subsystem-real reflection words, dedup sorted by `gamma_lambda` (`y_less`, blocks.cpp:727-728), assigning y = 0..count (blocks.cpp:790-842).
- Step 4: BFS over involution packets (queue of packet boundaries, `x_seen` BitMap): per generator s, if cross leads to unseen involution create whole new packet (x rows × the cross-imaged y list) (blocks.cpp:886-946); cross links via hash `srm_hash.find(ctxt.cross(s,srm))` (948-956); descent statuses set per case (958-982); real-parity Cayleys: `down_Cayley` collects new y models; if target involution unseen, complete the x-fiber over `sample_x` by imaginary-simple reflections (985-1034); Cayley links both directions, type1 gets the second image via cross of the down-Cayley (1036-1059). Lengths counted downward then reversed: `entry.length = max_length - entry.length` (1069-1075). Finally `sort()` by (length,x,y) (1077, 1488-1517) and `entry_element = lookup(srm)` (1079).
- Partial-block ctor takes a precomputed element list; per element computes status/links directly with `lookup`, links may stay `UndefBlock` (blocks.cpp:1086-1248).

## 3. Per-element data / DescentStatus

- `DescentStatus::Value` enum, order significant: `{ ComplexAscent, RealNonparity, ImaginaryTypeI, ImaginaryTypeII, ImaginaryCompact, ComplexDescent, RealTypeII, RealTypeI }` — first 4 ascents, last 4 descents; `isDescent` = bit 2 (`0x4`); `isDirectRecursion` = mask `0x5` (ComplexDescent|RealTypeI) — gkmod/descents.h:40-50,71-73. Stored one octet per simple root, `RANK_MAX` array (descents.h:44). Dual table at descents.h:74-79.
- Status computation for classic Block (`descents(x,y,kgb,dual_kgb)`, blocks.cpp:1541-1568): complex → C±(by `kgb.isDescent`); imaginary-noncompact at x → `kgb.cross(s,x)!=x` ? i1 : i2; else dual-imaginary-noncompact at y → `dual_kgb.cross(s,y)!=y` ? r2 : r1 (note the inversion vs x-side); else x-real → rn; else → ic.
- `common_block` uses `ctxt.status(s,x)` returning `(gradings::Status::Value, bool)` (repr.h:671; gradings enum `{Complex, ImaginaryCompact, Real, ImaginaryNoncompact}` structure/gradings.h:58) plus `is_parity` on the srm; mapping at blocks.cpp:958-982 (real: parity→r1/r2 by type-1 bool, else rn).
- Semantics: ic counts as (weak) descent, rn as ascent, but neither is *strict* (blocks.cpp:113-120; isStrictAscent/Descent blocks.cpp:272-290). `descent_generators` = set of weak descents (blocks.cpp:196-202).
- Accessors: `x(z)`, `y(z)`, `length(z)`, `descentValue(s,z)`, `cross`, `cayley`/`inverseCayley` (blocks.h:129-169). Bruhat order available on demand: `bruhatOrder()` fills via `complete_Hasse_diagram` (blocks.h:198-199; blocks.cpp:385-390, 1609-1653 — first strict good descent, complex or RealTypeI, else gather RealTypeII inverse Cayleys; `insert_ascents` closure at 1576-1597).

## 4. Membership condition

- (x,y) ∈ block ⟺ `dual_involution(involution(x))` equals the dual-KGB involution of y, i.e. y ranges exactly over `dual_kgb.tauPacket(dual_w[i])` while x ranges over `kgb.tauPacket(w_i)` (blocks.cpp:548-558). No further filter — the fibred product over matched involutions IS the block.
- `dual_involution`: for τ = w·δ, find v with −τ^t = v·δ∨; characterised by f(e)=w₀, f(s·w)=f(w)·dwist(s); implemented as `result = w0; for letters of word(w) right-to-left: mult by dual twist` — blocks.cpp:1666-1711. Equivalent closed forms `w0 * dual_twist(w⁻¹)` noted at 1692-1696.
- Matrix-level statement: θ_y = −θ_x^t (blocks.cpp:44-46, blocks.h:262-263).
- Cardinality precomputed as Σ_w packet_size(w)·dual_packet_size(dual_w) — blocks.cpp:537-543.

## 5. `block` command output format (differential surface)

Row per element z, from `do_print` (io/block_io.cpp:54-110), classic Block (`traditional==true`, line 66):

```
z(x,y):  length  [desc,desc,...]  cross...  (c1,c2)...  Cartan#  involution
```

- `z` width = digits(size−1); then `'(' x ',' y "):"` (block_io.cpp:71-76, only for `Block`).
- length, width padded (79).
- descents: `printDescent` → `[..]` comma-separated 2-char codes: `C-` ComplexDescent, `C+` ComplexAscent, `ic`, `rn`, `i1`, `i2`, `r1`, `r2` (block_io.cpp:373-419); masked entries print `* `.
- cross actions per s: number or `*` if Undef (85-91).
- Cayley column per s: prints `inverseCayley` if weak descent else `cayley`, as `(a,b)` with `*` for Undef (93-103).
- Suffix from `Block::print` (113-126): Cartan class number + root-datum involution — as reduced-involution expression if `as_invol_expr` (command `blockd`, blockmode.cpp:378-382) else as Weyl word (plain `block`, blockmode.cpp:326-330).
- `common_block::print` suffix instead: `*`/space (survives with singular), `(x=…,gamma-lambda=…)`, involution (block_io.cpp:128-147).
- Related commands sharing the surface: `smallblock`, `dualblock`, `smalldualblock`, `dualmap` (prints `dual_map` bijection z ↦ dual_block.element(y(z),x(z)), blocks.cpp:1715-1724; blockmode.cpp:332-375), `blocku`, `blockorder`.

## Porting notes (asymmetries worth flagging)

- y-side uses `dual_kgb.inverseCayley` where x-side uses `kgb.cayley` (blocks.cpp:575-590); i1/i2 vs r2/r1 cross-fixed-point tests are mirrored (blocks.cpp:1551-1560).
- Block stores per-(s,z) a single `Cayley_image` pair doubling as forward AND inverse Cayley, disambiguated by descent status (blocks.h:140-155) — a Rust port can keep one table + status test.
- Classic Block ordering: by involution packet (KGB order), x-major then y-consecutive, enabling O(1) `element(x,y)`; common_block instead sorts by (length,x,y) and looks up by hash of StandardReprMod.

---

# KL polynomial computation — oracle notes (atlasofliegroups @ 4d3e9449)

Files: `sources/gkmod/klsupport.{h,cpp}`, `sources/gkmod/kl.{h,cpp}` (no kl_error file exists; errors via `sources/error/error.h`). All paths below relative to `/Users/hoxide/mycodes/atlasofliegroups/sources/`.

## 1. KLSupport (klsupport.{h,cpp})

Derives from block only; owned per `KL_table` (KL_table inherits it, kl.h:68-69).

**Per-element info** (`Elt_info`, klsupport.h:35-41):
- `descents` = RankFlags: s set iff `DescentStatus::isDescent(descentValue(s,z))` (klsupport.cpp:81-83). Definition of descent: bit 2 of enum value — descents are {ImaginaryCompact, ComplexDescent, RealTypeII, RealTypeI} (descents.h:40-41, 47, 71).
- `good_ascents` = s that are neither descents nor ImaginaryTypeII, i.e. {ComplexAscent, ImaginaryTypeI, RealNonparity} (klsupport.cpp:84-85; comment 70-73).

**length_stop**: `length_stop[l]` = first BlockElt of length ≥ l = number of elements of length < l (klsupport.cpp:45-63). Accessors `length_less(l)`, `length_floor(y)=length_stop[length(y)]` (klsupport.h:65-67).

**Extremal/primitive definitions** (klsupport.h:118-121):
- `is_extremal(x, desc_y)` ⇔ `descent_set(x).contains(desc_y)` (every descent of y is a descent of x).
- `is_primitive(x, desc_y)` ⇔ `(good_ascent_set(x) & desc_y).none()` (every descent of y is descent or i2 ascent for x).

**Primitivization table** `d_prim_index` indexed by descent-set-as-ulong, 2^rank slots (klsupport.h:44-50, klsupport.cpp:43):
- `fill_prim_index(A)` (klsupport.cpp:109-144): backwards loop over x. If x primitive for A, its index recorded; else take first good ascent s in A: if `descent_value(s,x)==RealNonparity` → dead_end (means P_{x,y}=0 for all y with that descent set, klsupport.cpp:95-98); else follow `unique_ascent(s,x)` (C+ cross image or i1 first Cayley, blocks.h:157-164); UndefBlock link (partial block) → dead_end. Final pass reverses indices so primitives are numbered increasing; dead_end maps to `range` = number of primitives (klsupport.cpp:139-143).
- `prim_index(x,A)` returns index of primitivized x among primitives for A; `x==UndefBlock` → `range` (klsupport.h:102-106). `self_index(y)=prim_index(y,descent_set(y))` (klsupport.h:115-116).
- Lazy: `prepare_prim_index(A)` fills once per descent set (klsupport.h:95-100), called at start of each column fill (kl.cpp:353).
- Old pointwise `primitivize` is `#if 0`-disabled, replaced by table (klsupport.cpp:148-181).

**Traversal helpers**: `extr_back_up` / `prim_back_up` decrement x to previous extremal/primitive (klsupport.h:130-139); `col_size(y)` = # primitives for desc(y) of length < length(y) (klsupport.h:142-145).

**Ascent finders**: `ascent_descent(x,y)` = first bit of `descent_set(y)-descent_set(x)` (klsupport.h:82-83); `good_ascent_descent(x,y)` = first bit of `good_ascent_set(x)&descent_set(y)` (klsupport.h:86-87).

## 2. The recursion (kl.cpp)

Column-per-y fill: `fill_KL_column` (kl.cpp:350-363): `prepare_prim_index(descent_set(y))`, then if `first_direct_recursion(y) < rank()` use direct recursion, else `new_recursion_column`.

**Direct recursion applies** iff y has a descent s with `isDirectRecursion(d[s])` = ComplexDescent or RealTypeI (kl.cpp:249-259; mask descents.h:50,72-73). Then sy = `cross(s,y)` (complex) or `inverseCayley(s,y).first` (real type I) (kl.cpp:386-389).

`recursion_column` (kl.cpp:381-450) loops over *extremal* x < length_floor(y), increasing (kl.cpp:391-399), switch on `descent_value(s,x)`:
- ImaginaryCompact: `P_{x,y} = (q+1)P_{x,sy}` (kl.cpp:406-411)
- ComplexDescent: `P_{sx,sy} + q·P_{x,sy}` (kl.cpp:412-418)
- RealTypeI: `P_{sx.first,sy} + P_{sx.second,sy} + (q-1)P_{x,sy}` (add q·P, subtract P last) (kl.cpp:419-427)
- RealTypeII: `P_{sx,sy} + q·P_{x,sy} - P_{s×x,sy}` (sx = inverseCayley first; s×x = cross) (kl.cpp:429-435)

then subtracts mu correction `mu_correction` (kl.cpp:447-448, 480-525): subtract `∑_z mu(z,sy)·q^{(l(y)-l(z))/2}·P_{x,z}` over z with nonzero μ(z,sy) in `d_mu[sy]` and s a descent for z (kl.cpp:487-494); loops z decreasing, inner loop over extremals x with l(x)<l(z), separate exact leading-term subtraction when x==z (kl.cpp:515-521). Formula comment: `lhs = c_s.c_{y'} − ∑_z mu(z,y')c_z` (kl.cpp:461-469).

`complete_primitives` (kl.cpp:544-589) then walks primitives backwards; extremal x take the klv polynomial; primitive-non-extremal x have an i2 ascent s=`ascent_descent(x,y)` and get `P_{x,y} = P_{x',y}+P_{x'',y}` with (x',x'')=cayley(s,x), looked up in the current column (kl.cpp:566-574).

**New recursion / endgame** (`new_recursion_column`, kl.cpp:637-791) — used when y has no C- and no r1 descents (only r2 true descents; comment kl.cpp:598-601). Column allocated over all primitives (+1 for y itself, `P_{y,y}=1` at `self_index`) (kl.cpp:644-645). μ-list seeded with down-set of y, each μ=1 (kl.cpp:649-653; down_set = elements reached by C-, r1 (both inverse Cayleys), r2 (first) descents, blocks.cpp:204-229). Reverse loop over primitives x (kl.cpp:662):
1. x primitive-not-extremal: same i2 sum `P = P_{x',y}+P_{x'',y}` — "equation (1.9) in recursion.pdf" (kl.cpp:666-672).
2. Else find s = `first_nice_and_real(x,y)`: s rn for y and one of {C+, i2, ic} for x (kl.cpp:270-286). Start from `Pxy = mu_new_formula(x,y,s,mu_pairs)` (= expression (3.4)), then:
   - C+ for x: subtract `q·P_{sx,y}` (eq (3.3a)=(3.4)) (kl.cpp:691-693)
   - i2 for x: add `(1−q)(P_{x',y}+P_{x'',y})` then `safeDivide(2)` (eq (3.5)) (kl.cpp:695-703)
   - ic for x: `safe_quotient_by_1_plus_q(l(y)−l(x))` (eq (3.7): recover P from (q+1)P truncated) (kl.cpp:706-711)
   New μ recorded if `l_y == l_x + 2·deg + 1` (kl.cpp:716-717).
3. Endgame: `first_endgame_pair(x,y)` (kl.cpp:317-340): pair (s,t) with (s,t)=(rn,r2) for y, s=i1 for x, t∈{i1,i2} for s×x. Then `Pxy = mu_new_formula(...) + (1−q)P_{x',y}` (x'=cayley(s,x).first; formula (3.6)) giving `P_{x,y}+P_{s.x,y}`; subtract `P_{s.x,y}` computed via t: `P = Σ KL_y(cayley(t, cross(s,x)))` (kl.cpp:742-767). If s found but s×x outside partial block, t search skipped, `P_{s.x,y}=0` (kl.cpp:328-330, 756). If no pair: `P_{x,y}=0` (kl.cpp:769-770).

`mu_new_formula` (kl.cpp:813-841): `∑_{x<z<y, s descent for z} mu(z,y)·q^{(l(y)-l(z)+1)/2}·P_{x,z}` (comment kl.cpp:801-805), loop over mu_pairs decreasing in length, stop at `l(z)<=l(x)`.

**Fill order**: `silent_fill` iterates holes (=unfilled y) in increasing BlockElt order (block is length-sorted) (kl.cpp:844-864); `verbose_fill` does the same grouped by length for progress reporting (kl.cpp:867-951). Within a column: extremal x increasing (direct case), then primitives backwards; new-recursion case: primitives strictly decreasing.

## 3. Storage

- `KLStore = PosPolEntry::Pooltype` = `std::vector<Safe_Poly<KLCoeff>>` (kl.h:50); `KLPol = Safe_Poly<KLCoeff>` (Atlas.h:461). Constructed containing `{Zero, One}` at indices 0,1 (`enum {zero=0, one=1}`, kl.h:86; kl.cpp:100).
- `d_KL[y]` = `KL_column = std::vector<KLIndex>`, one entry per primitive x of length < l(y) for desc(y), position = `prim_index` (kl.h:47, 74-75; sizing kl.cpp:548, 644/777). Lookup `KL_pol(x,y)`: primitivize via `prim_index`; out-of-column index → One if `self_index(y)` else Zero (kl.cpp:124-132).
- Dedup: `KL_hash_Table = HashTable<PosPolEntry, kl::KLIndex>` (Atlas.h:467-468); each computed polynomial stored via `hash.match(P)` (kl.cpp:561, 573, 779). Hash function: from top coefficient down, `h = (h<<21)+(h<<13)+(h<<8)+(h<<5)+h+P[i]`, masked by modulus−1; zero → 0 (polynomials.h:199-207, hashCode at 256-257). Table may be shared/external (`pol_hash` ctor arg) or owned (`own`, initial size arg 4) (kl.h:80-83, 58-61; kl.cpp:94-104, 177-180).
- **mu**: `Mu_pair {BlockElt x; MuCoeff coef}`, `d_mu[y]` = vector of pairs sorted increasing by x, only nonzero μ (kl.h:41-48, 76-78). Two sources (comment kl.cpp:1023-1028): (a) extremal x with `l(y)==l(x)+2·deg(P)+1`, μ = leading coeff (kl.cpp:562-564, 716-717, 766-767); (b) down-set x, μ=1 without computing P (kl.cpp:578-585, 650-653). Accessor `mu(x,y)` recomputes from `KL_pol` degree instead of searching d_mu (kl.cpp:150-154).

## 4. Entry points & laziness

- `Block_base::kl_tab(pol_hash, limit=0, verbose)` → `fill_kl_tab` → constructs `kl::KL_table` first time, then `fill(limit,verbose)` (blocks.h:200-203, blocks.cpp:392-400).
- Commands (blockmode.cpp): `klbasis` (463-471), `kllist` (475-481), `primkl` (491-499), `klwrite` (502-523), plus `wcells`/`wgraph` via `wGraph` (526-539, kl.cpp:1042-1058). All call `currentKL()` = `currentBlock().kl_tab(nullptr,0,true)` — full fill, verbose=true (blockmode.cpp:173-176); reprmode same for common blocks (reprmode.cpp:145).
- `KL_table::fill(limit,verbose)` (kl.cpp:188-221): limit==0 → size(); returns early if `limit<=first_hole()`; partial fill supported via `d_holes` BitMap + `plug_hole` (kl.h:72, 136; kl.cpp:182-185). Verbose forced off unless compiled with `-DVERBOSE` (kl.cpp:195-197). `bad_alloc` → clears partially written columns, rethrows `error::MemoryOverflow` (kl.cpp:210-219).
- Verbose output (stderr): "computing Kazhdan-Lusztig polynomials ..." (kl.cpp:203); per y `y\r` (kl.cpp:896); per completed length `l=%3d, y=%6d, polys:%11d, mat:%11d` then CPU/mem line (kl.cpp:912-932); final totals "Total elapsed time = %gs.", "N polynomials, M matrix entries." (kl.cpp:936-941).

## 5. Output surfaces (io/kl_io.cpp, io/filekl.cpp)

**klbasis** → header "Full list of non-zero Kazhdan-Lusztig-Vogan polynomials:\n\n" (blockmode.cpp:468-469), then `printAllKL` (kl_io.cpp:48-94): for each y (all y, 0..size-1): `setw(width) y << ": "`, then for each x ≤ y with nonzero P: first on same line `setw(width) x << ": " << poly`, subsequent lines indented `setw(width+2)` blank + `setw(width) x << ": " << poly` (kl_io.cpp:59-77); blank line between y's; trailer `"<count> nonzero polynomials, and <comp-count> zero polynomials,\n at <comp> Bruhat-comparable pairs."` using `block.bruhatOrder().n_comparable()` (kl_io.cpp:83-91). width = decimal digits of size−1, tab=2 (kl_io.cpp:53-54).

**kllist** → `printKLList` (kl_io.cpp:155-173): all distinct nonzero store polynomials, sorted by `polynomials::compare`, one per line.

**primkl** → header "Kazhdan-Lusztig-Vogan polynomials for primitive pairs:\n\n" (blockmode.cpp:496-497), `printPrimitiveKL` (kl_io.cpp:98-151): per y, all primitive x with `Bruhat.lesseq(x,y)` printed (including zero ones), same indent shape, then extra line for `y: 1` and trailer counting Bruhat-comparable primitive pairs / null polys / incomparable pairs (kl_io.cpp:143-148).

Polynomial print format (`Polynomial::print`, indeterminate "q", kl_io.cpp:34): monomials from high degree down, "+" separators, coefficient omitted when 1, "q^d" for d>1, "q" for d=1, bare coeff for d=0, "0" for zero (polynomials_def.h:299-331).

**klwrite** → two binary files (blockmode.cpp:502-523): `write_matrix_file` (filekl.cpp:101-119): per y a row = `put_int(y)`, #primitives+1, bitmap of nonzero positions in 32-bit words, nonzero KLIndex values only, final `put_int(1)` for P_{y,y} (filekl.cpp:66-99); then row-size delta table; magic code overwrites first 4 bytes. `write_KL_store` (filekl.cpp:121-152): count, 5-byte offsets per polynomial, then 4-byte coefficients per polynomial.

**printMu** exists (`kl_io.cpp:177-193`): per y, comma-separated `(x,coef)` pairs.

## 6. Coefficient type & overflow

- `KLCoeff = unsigned int`; `MuCoeff = KLCoeff`; `KLIndex = unsigned int` ("<2^32 distinct polynomials for E_8!") (Atlas.h:460-463).
- All arithmetic through `Safe_Poly`: `safe_add`/`safe_multiply` test against `numeric_limits<C>::max()` and throw `error::NumericOverflow` (polynomials_def.h:346-375); `safe_subtract` only asserts (mathematical non-negativity guaranteed) (polynomials_def.h:353-360); `safeDivide` asserts exactness (377-382); `safe_quotient_by_1_plus_q` divides by q+1 imagining a μ·q^{d+1} leading term, asserts degree bound (polynomials_def.h:512-527). `Safe_Poly::safeAdd/safeSubtract` handle aliasing by looping top-down (polynomials_def.h:392-478).
- `NumericOverflow` is a plain struct (error.h:61-63); fill catches it and rethrows `std::runtime_error("Numeric overflow in KL computations")` (kl.cpp:860-863, 946-949). `bad_alloc` → `error::MemoryOverflow` with column rollback (kl.cpp:210-219).

## Key invariants worth porting

- Degree bound checked everywhere: P_{x,y}=0 or `2·deg < l(y)−l(x)`, with equality `2·deg+1 == l(y)−l(x)` exactly at μ-contributing pairs (assert kl.cpp:441-444).
- Subtractions must come after additions in each case ("subtraction must be last", kl.cpp:426, 434) since coefficients are unsigned.
- Zero polynomial = size-0 vector (kl.cpp:79-80); One = `Polynomial(1)` (kl.cpp:83).

---

# Dual-side machinery for blocks — oracle notes (atlasofliegroups @ 4d3e9449)

## 1. Dual inner class / dual real form → block

### Dual root datum
- `RootDatum(const RootDatum& rd, tags::DualTag)` swaps roots↔coroots, weight↔coweight numerators, radical↔coradical bases, 2ρ↔2ρ^∨ — `sources/structure/rootdata.cpp:877-896`. Root `i` of the dual datum is coroot `i` of the original (ordering preserved, "users should not depend on this") — `rootdata.cpp:872-875`. Also `PreRootDatum(prd, DualTag)` — `sources/structure/prerootdata.h:63`.

### Dual distinguished involution
- `dualBasedInvolution(q, rd) = (q * rd.action_matrix(w0)).negative_transposed()`, i.e. **δ^∨ = (-q·w0)^t = (w0^t)(-q^t)**; w0 makes the dominant chamber dominant again; -w0 is central in based-datum automorphisms so left/right multiplication is equivalent — `sources/structure/rootdata.cpp:1275-1288` (decl `rootdata.h:41`).
- Every `InnerClass` already carries the dual side internally: `own_dual_datum(new RootDatum(*own_datum,DualTag))` — `sources/structure/innerclass.cpp:128`; dual fundamental fiber `d_dualFundamental(d_dualRootDatum, dualBasedInvolution(tmp_d,d_rootDatum))` — `innerclass.cpp:138`; dual Tits group `d_dualTitsGroup(d_dualRootDatum, W, dualDistinguished())` — `innerclass.cpp:142`. Accessors: `dual_root_datum()` `innerclass.h:264`, `dualDistinguished()` = involution of dual fundamental fiber `innerclass.h:326-327`.

### Dual InnerClass constructor (`tags::DualTag`)
- `InnerClass(const InnerClass& G, tags::DualTag)` — `sources/structure/innerclass.cpp:403-480`; comment "Construct the complex reductive group dual to G (used in Fokko only) ... they can share |W|" — `innerclass.cpp:401-402`.
  - Swaps the stored pair: `d_rootDatum(G.d_dualRootDatum), d_dualRootDatum(G.d_rootDatum)` — `innerclass.cpp:405-406`; shares W (`my_W(nullptr), W(G.W)`) — `innerclass.cpp:408`; swaps fundamental fibers `innerclass.cpp:412-413` and Tits groups `innerclass.cpp:415-416`; Cartan poset dualized `innerclass.cpp:420`.
  - Cartan classes: same set, **reverse order**, dual twisted involution obtained by `tw_org = W.prod(src.tw, W.longest())` then `canonicalize` — `innerclass.cpp:432-440`.
  - **Real-form numbering swap**: `dst.real_forms = src.dual_real_forms; dst.dual_real_forms = src.real_forms;` plus `rep`/`dual_rep` swap — `innerclass.cpp:443-446`. So *real form number `drf` of the dual inner class ≡ dual real form number `drf` of G*.
- Dual TwistedWeylGroup: same W, twist replaced by `dual_twist()`: `twist[s] = W.Chevalley_dual(twisted(s))` (Chevalley dual = conjugation by w0 composed with twist) — `sources/structure/weyl.cpp:1227-1233, 1239-1242`.

### Pairing rf × drf → block (argument flow)
- Fokko `block` command: mainmode caches `dual_ic_pointer = new InnerClass(current_inner_class(), tags::DualTag())` — `sources/interface/mainmode.cpp:105-108`. Block-mode entry: `drf = interactive::get_dual_real_form(...); dual_G_R_pointer = new RealReductiveGroup(current_dual_inner_class(), drf)` — `sources/interface/blockmode.cpp:211-214`; block built as `new Block(Block::build(currentRealGroup(), currentDualRealGroup()))` — `blockmode.cpp:168`.
- Compatibility restriction: `get_dual_real_form` offers only `drfl = G.dualRealFormLabels(G.mostSplit(rf))` — dual real forms defined at the most-split Cartan of rf — `sources/io/interactive.cpp:511-514`.
- Self-contained pseudo-constructor: `Block::build(InnerClass& G, RealFormNbr rf, RealFormNbr drf)` does `RealReductiveGroup G_R(G,rf); InnerClass dG(G,tags::DualTag()); RealReductiveGroup dG_R(dG,drf); KGB kgb(G_R, common_Cartans(G_R,dG_R)); KGB dual_kgb(dG_R, common_Cartans(dG_R,G_R)); return Block(kgb,dual_kgb);` — `sources/gkmod/blocks.cpp:610-619`. Variant from two prebuilt real groups (full KGBs) — `blocks.cpp:622-626`.
- `common_Cartans(GR,dGR) = GR.Cartan_set() & GR.innerClass().dual_Cartan_set(dGR.realForm())` — `blocks.cpp:1727-1728`; `dual_Cartan_set(drf)` flags Cartans whose `dual_real_forms` contain drf — `innerclass.cpp:685-694`.

## 2. y-side objects

### Classic `Block` (Fokko): y IS a KGB element of the dual group
- `Block::Block(const KGB& kgb, const KGB& dual_kgb)` — `blocks.cpp:526-606`. Dual KGB is built by **the same `KGB` class** on `RealReductiveGroup(dual_inner_class, drf)` (`KGB(RealReductiveGroup&, const BitMap& Cartan_classes)` — `sources/gkmod/kgb.h:313`); nothing dual-specific in the KGB code, only its inputs.
- Element = pair `(x,y)`: `struct EltInfo { KGBElt x,y; ... }` "indices into |KGB| sets (which might no longer exist)" — `sources/gkmod/blocks.h:62-80`; hash `(13*x+21*y)` `blocks.h:77`.
- Fibred product over matched twisted involutions: for each involution `w` of kgb, `dual_w = dual_involution(w,tW,dual_tW)`; size += `kgb.packet_size(w)*dual_kgb.packet_size(dual_w)`; elements are `x` in `kgb.tauPacket(w)` × `y` in `dual_kgb.tauPacket(dual_w)` — `blocks.cpp:535-558`.
- Links: cross = `element(kgb.cross(s,x), dual_kgb.cross(s,y))`; Cayley on x pairs with **inverseCayley on y** — `blocks.cpp:570-590`. Descent status from the two KGB statuses (x complex/imaginary decides; else y ImaginaryNoncompact ⇒ Real type; else RealNonparity / ImaginaryCompact) — `blocks.cpp:1541-1568`; status compatibility discussion `blocks.cpp:48-111`.

### Modern `common_block` / repr: y is a torus-element residue, NOT a dual KGB element
- Conceptual type `TorusElement`: rational weight `repr` modulo 2Z, meaning exp(πi·repr), "elements of the torus of the dual group" — `sources/structure/y_values.h:26-45`; equality/order componentwise on normalized `RatWeight` — `y_values.h:61-63`. (`y_entry` is only a vestigial forward declaration, `sources/Atlas.h:335,338` — no definition or use anywhere else.)
- `StandardRepr` stores `KGBElt x_part; TorusPart y_bits; RatWeight infinitesimal_char` — `sources/gkmod/repr.h:74-110`; `y_bits` is the torsion part of λ, "would be represented in the |Block| structure by the |TorusPart| component of the |TitsElt| of the dual KGB-element (y)"; it represents an element of **(X^*)^{-θ} / (1-θ)X^*** — `repr.h:51-58`. Hash: `x + 375*y_bits + 83*denom(γ)` folded with numerator — `sources/gkmod/repr.cpp:43-50`.
- Packing (InvolutionTable): `y_pack(inv,λ-ρ) = TorusPart(M_real * λρ)` (coords on (1-θ)X^* basis mod 2) — `sources/structure/involutions.h:210-212`; inverses `y_lift`/`y_unlift` `involutions.h:214-215,245-250`; canonical representatives `lambda_unique`, `real_unique` — `involutions.h:199-203`.
- `StandardReprMod` (block-construction element): `{KGBElt x_part; RatWeight gamlam}` where `gamlam = real_unique(γ-λ)`; the λ=ρ shift trick makes `y_bits` unnecessary — `repr.h:112-131`; built via `mod_reduce`/`build` which apply `involution_table().real_unique(inv_nr(x), γ-λ)` — `repr.cpp:52-66`. Equality is plain `(x, gamlam)` equality since representatives are canonical — `repr.h:143-145`; hash `x + 47*denom` folded with numerator — `repr.cpp:68-73`.
- Inside `common_block`, "y" is just a consecutive numbering of distinct `gamma_lambda` values per involution packet: elements hashed in `srm_hash` (`z_pool` of `StandardReprMod`) — `blocks.h:351-354`; y counter `y_count` and packet invariant "the |y|s are consecutive" — `blocks.cpp:788, 834-841, 869-881`.

## 3. Involution correspondence θ_x ↔ θ_y

- Exact condition: with τ = w·δ on X^* (x-side) one seeks the twisted involution v in dual tW with **-τ^t = v·δ^∨** acting on X_*, where δ^∨ acts as minus-transpose of w0·δ — `blocks.cpp:1666-1672`.
- Word-level bijection f: W→W characterized by **f(e) = w0, f(s·w) = f(w)·dwist(s)** (dwist = dual twist); it intertwines twisted conjugations: f(s·w·twist(s)) = s·f(w)·dwist(s) — `blocks.cpp:1673-1677`.
- Implementation `dual_involution(w,tW,dual_tW)`: start from `dual_W.longest()`, right-multiply by `dual_tW.twisted(ww[i])` for the letters of `word(w)` right-to-left — `blocks.cpp:1701-1711`. Only assumes shared *external* generator numbering — `blocks.cpp:1682-1685`. With shared underlying W it equals `prod(w0, dual_tW.twisted(W.inverse(w)))` — `blocks.cpp:1690-1696`.
- Same relation at Cartan-class level in the dual InnerClass: dual twisted involution rep = canonicalize(src.tw · w0) — `innerclass.cpp:436-438`.
- Duality of blocks at element level: `dual_map(b, dual_b)[i] = dual_b.element(b.y(i), b.x(i))` (x and y swap) — `blocks.cpp:1715-1724`.

## 4. Minimal dual-side machinery for the port

Given: arbitrary based root datum + arbitrary distinguished involution + KGB per real form — **yes, the dual block side is exactly the existing pipeline re-instantiated**; the oracle's dual KGB is the unmodified `KGB` class run on `RealReductiveGroup(InnerClass(G,DualTag), drf)` (`blocks.cpp:610-619`). Needed pieces:

1. **Dual datum**: swap roots↔coroots (`rootdata.cpp:877-896`). Keep the original ordering (dual root i = coroot i) if you want oracle-matching numbering, but nothing downstream depends on it (`rootdata.cpp:872-875`).
2. **Dual distinguished involution**: **δ^∨ = -(δ·w0)^t** (`rootdata.cpp:1283-1288`) — yes, precisely "-w0·δ transposed"; sides interchangeable since -w0 is central among based automorphisms (`rootdata.cpp:1280-1282`).
3. **Weyl group**: may be rebuilt or shared; if shared, the dual twist is s ↦ Chevalley_dual(twist(s)) = w0-conjugate of the twist (`weyl.cpp:1227-1242`). The only cross-side requirement for `dual_involution` is consistent external generator numbering (`blocks.cpp:1682-1685`).
4. **Form pairing**: real form numbers of the dual inner class *are* G's dual-real-form numbers, by construction `dst.real_forms = src.dual_real_forms` (`innerclass.cpp:443-444`); dual quasisplit is number 0 (`innerclass.cpp:596` assert, and dual labels anchored at the all-noncompact dual grading in `map_dual_real_forms`, `innerclass.cpp:566-598`). A pair (rf, drf) is block-valid iff drf ∈ `dualRealFormLabels(mostSplit(rf))` (`interactive.cpp:511-514`), equivalently `Cartan_set(rf) ∩ dual_Cartan_set(drf) ≠ ∅` (`blocks.cpp:1727-1728`, `innerclass.cpp:674-694`). The per-Cartan label lists are `realFormLabels/dualRealFormLabels` indexed by weak-real partition orbits of the (dual) fiber (`innerclass.h:448-461`).
5. **Block assembly needs only**: per-involution packets on both KGBs (`tauPacket`, `packet_size`), the `dual_involution` word map, both cross actions, Cayley on x + inverseCayley on y, and the status table match (`blocks.cpp:535-606, 1541-1568`). Optional efficiency: restrict each KGB to common Cartans (`blocks.cpp:616-617`); the full-KGB variant works unrestricted since non-matching involution packets pair with empty ones (`blocks.cpp:622-626`).

Caveat: the modern parameter/`common_block` route (`repr.h`, `blocks.cpp:735-`) needs **no dual KGB at all** — y is absorbed into `gamlam = real_unique(γ-λ)` with `y_pack/y_lift/real_unique` from the existing InvolutionTable (`involutions.h:199-215`); only the classic Fokko `Block` (the natural differential-test target against `block` command output) needs the dual-KGB instantiation above.
