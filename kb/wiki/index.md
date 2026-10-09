# Knowledge Wiki

## Concepts

- **[[dual-预热历史与-weyl-群兼容性|dual 预热历史与 Weyl 群兼容性]]** — canonical dual 仅在目标尚未建立 WeylGroup 时共享源群身份，已预热目标保持独立，因此 Weyl 元素兼容性取决于对象生命周期和构造历史。
- **[[innerclass-对偶构造与生命周期保持|InnerClass 对偶构造与生命周期保持]]** — original InnerClass 构造立即取得 canonical dual 并强持有 primal 与 dual，Rust 对齐需覆盖该隐式对偶路径及其生命周期，显式 dual 修复不足以证明完整兼容。
- **[[mod-2-投影与纤维商上的诱导映射|mod-2 投影与纤维商上的诱导映射]]** — mod-2 投影利用根系数的奇性翻转坐标位，FiberToAdjoint::apply 依次取规范代表、执行投影并构造目标纤维元素，每次应用现算而不缓存稠密矩阵。
- **[[parabolicpieces-的抛物分解与排序键|ParabolicPieces 的抛物分解与排序键]]** — ParabolicPieces 用内部层级的最小右陪集代表元分解生成 piece 索引列表，其字典序复现上游 WeylElt 排序并供 involution 排序及 KGB 重编号使用。
- **[[rootdatum-弱驻留与规范活对象身份|RootDatum 弱驻留与规范活对象身份]]** — original Atlas 按完整 PreRootDatum 内容及 preference 弱驻留 RootDatum，以存活对象的指针身份实现相等性；重型对象可释放，但轻量索引键仍保留。
- **[[rust-weyl-内核与抽象群的无环所有权模型|Rust Weyl 内核与抽象群的无环所有权模型]]** — 分离 datum 坐标内核与抽象 Weyl 群身份可表达共享语义并避免 handle 与 context 的强引用环；核心身份修复已落地，后续缓存共享设计仍需独立验证。
- **[[weyl-上下文共享的性能与内存证据边界|Weyl 上下文共享的性能与内存证据边界]]** — 重复上下文构造和采样热点支持优化调查，但构建次数假设与探针计时不证明加速；共享可能延长内核存活，需独立测量构建数、分配及 time/CPU/RSS。
- **[[weyl-元素的可失败关系与跨坐标运算|Weyl 元素的可失败关系与跨坐标运算]]** — Weyl 元素的等于、不等于和乘法须先检查群身份，包括 no_value 路径；兼容但坐标不同的值通过在左侧重放右侧外部生成元词运算，乘积保留左 owner。
- **[[weyl-元素的扭曲共轭|Weyl 元素的扭曲共轭]]** — twisted_conjugate 计算 s_gen·w·s_twist(gen)，要求 twist 为生成子对合置换，其与 distinguished involution 的一致性由调用方保证。
- **[[weyl-群的矩阵作用与词级元素双层结构|Weyl 群的矩阵作用与词级元素双层结构]]** — WeylAction 表示携带根 datum 的全格矩阵作用，WeylElement 表示枚举根的置换，两层通过 action_permutation 与 from_action 桥接互查。
- **[[weyl-语义回归的递进验证门禁|Weyl 语义回归的递进验证门禁]]** — 以独立进程原版捕获和 BEFORE/fix/AFTER 回归验证语义；已落地 A1 验收仅覆盖限定输入，G2 非对称编号、B2/C2、操作数顺序及独占元素生命周期等仍需后续见证。
- **[[weylaction-的对偶全格作用|WeylAction 的对偶全格作用]]** — 在 character 与 cocharacter 全格上维护反射矩阵及复合作用，反射构造使用检查算术，但矩阵复合包含未经检查的 i32 收窄。
- **[[weylaction-的等值与-datum-身份语义|WeylAction 的等值与 datum 身份语义]]** — 源码 derive 等值逐字段比较且包含 datum 值，因此“相等即矩阵作用相等”的概述须限定于 datum 值相同的情形。
- **[[weylelement-的置换表示与长度下降不变量|WeylElement 的置换表示与长度下降不变量]]** — 以正向置换、逆向量和缓存长度支持常数时间长度及左右 descent 查询，乘法重算长度，而单一 ambient RootSystem 的一致性由调用方保证。
- **[[仅成功发布的惰性初始化|仅成功发布的惰性初始化]]** — 惰性 cell 只发布完整成功结果，构造失败后应允许重试，且不能缓存首次调用的 Diagnostic 或 SourceSpan，以保留当前调用的错误定位。
- **[[伴随-cartan-纤维的构建与下降验证|伴随 Cartan 纤维的构建与下降验证]]** — AdjointCartanFiber 在检查 datum、对合与预算后构造伴随半单商上的有限 F₂ 纤维，并通过 validate_induced_map 验证源纤维映射的下降条件；来源仅为结构性阅读，不代表数学验收。
- **[[伴随纤维映射的测试证据与覆盖边界|伴随纤维映射的测试证据与覆盖边界]]** — 九项测试锚定中心核、可加性、矩阵字面量、逐坐标基交织关系、出处拒绝、rank-33 动态秩及部分预算拒绝路径，但多个错误分支和接口直接调用仍未覆盖，本次阅读未运行测试。
- **[[伴随纤维的资源预算与可恢复错误|伴随纤维的资源预算与可恢复错误]]** — AdjointFiberBudget 限制整数格、持久条目和投影工作量，以受检算术检测溢出；运行期投影预算逐次独立检查，分配失败等情况返回显式错误。
- **[[余权坐标的投影出处绑定|余权坐标的投影出处绑定]]** — AmbientCoweight 与 AdjointCoweight 将坐标绑定到 Arc<AdjointProjectionModel>，以指针身份和坐标共同判等，并拒绝跨独立投影实例复用坐标。
- **[[基于左下降剥离的规范约化词|基于左下降剥离的规范约化词]]** — canonical_word 按 WeylInterface 的内部生成子序逐次剥离最小左 descent，获得该序下字典序最小的约化词，并检查每步长度恰减一。
- **[[带基数预算的-weyl-群作用枚举|带基数预算的 Weyl 群作用枚举]]** — enumerate_actions 通过 CompactWeyl 枚举并并行物化矩阵，使用显式基数预算；来源描述了字典序输出，但未提供排序测试或独立性能验证。
- **[[根对合诱导的伴随余权作用|根对合诱导的伴随余权作用]]** — 根像的单根坐标构成根作用矩阵；由于根作用已验证为对合，其对偶作用的逆转置等于转置，因而伴随余权作用由转置矩阵给出。
- **[[源余权格到伴随余权格的整数投影|源余权格到伴随余权格的整数投影]]** — 整数投影 Y → P∨ 按源单根顺序计算 pair(root, y) 作为目标坐标，保持可加性且可能具有中心核，因此不应视为同构。

_23 pages | Generated 2026-10-09T15:34:09.096Z_
