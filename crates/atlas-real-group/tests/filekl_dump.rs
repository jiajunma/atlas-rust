//! Dump driver for the HPC `filekl_diff` differential job
//! (`hpc/filekl_diff.sbatch`).
//!
//! `#[ignore]`d by default: a normal `cargo test` skips it. The job runs it
//! with `--ignored` and `FILEKL_DUMP_DIR` set; for each of a fixed set of
//! small blocks it writes `<name>.block`, `<name>.matrix`, `<name>.kl` via
//! the `filekl` writers, plus a `<name>.json` expectation
//! file with the block size, rank, and every KL polynomial's coefficient
//! list (constant term first, `[]` for the zero polynomial) keyed by block
//! element pair. The upstream `KLread` stand-alone tool then reads the
//! binary files back and `hpc/filekl_diff.py` compares semantically.
//!
//! Block choice: the quasisplit form (largest KGB) of the compact inner
//! class against the quasisplit form of the dual inner class, matching the
//! interpreter's `full_block_of` recipe (domain_builtins.rs:8128-8131).

use std::env;
use std::fs;
use std::io::Write as _;
use std::path::Path;

use atlas_real_group::{
    dual_inner_class, write_block_file, write_kl_store, write_matrix_file, AdjointFiberBudget,
    BasedRootDatum, BlockGraph, CartanClassification, CartanClassificationBudget,
    ExternalFormOrder, InnerClass, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget,
    KgbGraph, KlTable, LatticeInvolution, RealFormSeed, StrongRealClassification,
};

fn class_budget(weyl: usize) -> CartanClassificationBudget {
    CartanClassificationBudget::new(
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
        AdjointFiberBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            50_000,
            100_000,
        ),
        weyl,
        64,
        64,
    )
}

fn lattice_budget() -> IntegerLatticeBudget {
    IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
}

/// The KGB graph of the quasisplit weak real form (via the interpreter's
/// own external-order layer), with the involution table the graph was built
/// against.
fn quasisplit_graph(
    inner_class: &InnerClass,
    classification: &CartanClassification,
    strong: &StrongRealClassification,
) -> (KgbGraph, InvolutionTable) {
    let order = ExternalFormOrder::build(inner_class, classification).unwrap();
    let form = order.internal(order.quasisplit_external()).unwrap();
    let mut table = InvolutionTable::new(
        inner_class,
        InvolutionTableBudget::new(64, lattice_budget()),
    )
    .unwrap();
    let fundamental = classification.cartan_ids().next().unwrap();
    table.add_cartan(classification, fundamental).unwrap();
    let seed = RealFormSeed::build(
        inner_class,
        classification,
        strong,
        &table,
        form,
        &lattice_budget(),
        4_096,
    )
    .unwrap();
    let graph = KgbGraph::build(inner_class, classification, strong, &mut table, &seed).unwrap();
    (graph, table)
}

/// Escape-free JSON for the fixed expectation shape; written by hand to
/// avoid pulling serde into dev-dependencies for one driver.
fn write_expectation(
    path: &Path,
    name: &str,
    block: &BlockGraph,
    kl: &KlTable,
) -> std::io::Result<()> {
    let mut json = String::new();
    json.push_str(&format!(
        "{{\"name\":\"{name}\",\"size\":{},\"rank\":{},\"polynomials\":[",
        block.size(),
        block.rank()
    ));
    let mut first = true;
    for y in 0..block.size() {
        for x in 0..=y {
            let index = kl.kl_pol(x, y).unwrap();
            let polynomial = kl.pool().get(index).unwrap();
            let coeffs: Vec<String> = polynomial
                .as_slice()
                .iter()
                .map(|coefficient| coefficient.to_string())
                .collect();
            if !first {
                json.push(',');
            }
            first = false;
            json.push_str(&format!(
                "{{\"x\":{x},\"y\":{y},\"coeffs\":[{}]}}",
                coeffs.join(",")
            ));
        }
    }
    json.push_str("]}");
    let mut file = fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn dump_block(name: &str, cartan: Vec<Vec<i32>>, out_dir: &Path) {
    // Weyl-group budgets are hard caps; G2 needs 12, so run everything at 16.
    let weyl_budget = 16;
    let datum = BasedRootDatum::standard(cartan).unwrap();
    let involution = LatticeInvolution::identity(&datum).unwrap();
    let inner_class = InnerClass::new(datum, involution, weyl_budget).unwrap();
    let classification =
        CartanClassification::build(&inner_class, &class_budget(weyl_budget)).unwrap();
    let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
    let (graph, primal_table) = quasisplit_graph(&inner_class, &classification, &strong);

    let dual_class = dual_inner_class(&inner_class, weyl_budget, 64).unwrap();
    let dual_classification =
        CartanClassification::build(&dual_class, &class_budget(weyl_budget)).unwrap();
    let dual_strong = StrongRealClassification::build(&dual_classification, 4_096).unwrap();
    let (dual_graph, dual_table) =
        quasisplit_graph(&dual_class, &dual_classification, &dual_strong);

    let block = BlockGraph::build(
        &graph,
        &primal_table,
        &dual_graph,
        &dual_table,
        &dual_class,
        weyl_budget,
    )
    .unwrap();
    let mut kl = KlTable::new(&block).unwrap();
    kl.fill(0).unwrap();

    let mut block_file = fs::File::create(out_dir.join(format!("{name}.block"))).unwrap();
    write_block_file(&block, &mut block_file).unwrap();
    let mut matrix_file = fs::File::create(out_dir.join(format!("{name}.matrix"))).unwrap();
    write_matrix_file(&kl, &mut matrix_file).unwrap();
    let mut kl_file = fs::File::create(out_dir.join(format!("{name}.kl"))).unwrap();
    write_kl_store(kl.pool(), &mut kl_file).unwrap();
    write_expectation(&out_dir.join(format!("{name}.json")), name, &block, &kl).unwrap();

    eprintln!(
        "dumped {name}: block size {}, rank {}, pool {}",
        block.size(),
        block.rank(),
        kl.pool().len()
    );
}

#[test]
#[ignore = "HPC differential driver; needs FILEKL_DUMP_DIR and the KLread oracle"]
fn filekl_dump() {
    let out_dir = match env::var("FILEKL_DUMP_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("FILEKL_DUMP_DIR unset; nothing to dump");
            return;
        }
    };
    let out_dir = Path::new(&out_dir);
    fs::create_dir_all(out_dir).unwrap();

    dump_block("a1", vec![vec![2]], out_dir);
    dump_block("a2", vec![vec![2, -1], vec![-1, 2]], out_dir);
    dump_block("b2", vec![vec![2, -2], vec![-1, 2]], out_dir);
    dump_block("g2", vec![vec![2, -1], vec![-3, 2]], out_dir);
}
