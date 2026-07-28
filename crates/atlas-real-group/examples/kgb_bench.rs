//! KGB differential benchmark: build the full pipeline for a battery of
//! simply connected equal-rank inner classes and print, per weak real form
//! in the EXTERNAL (upstream output) order, the KGB size and the sorted
//! length multiset — the exact observables the upstream `atlas` binary
//! prints for the same groups — plus wall-clock timings per stage.
//!
//! Output lines are machine-parseable:
//!   `<group> form <external> size <n> lengths <l0>,<l1>,...`
//!   `<group> time_ms <pipeline> kgb_ms <all-forms>`
//! Run with group names as arguments, or no arguments for the default
//! battery.

use std::time::Instant;

use atlas_real_group::{
    AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget, Coweight,
    ExternalFormOrder, InnerClass, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget,
    KgbGraph, LatticeInvolution, RealFormSeed, StrongRealClassification, Weight,
};

/// Upstream `lietype.cpp` Cartan-entry convention (dispatch, lietype.cpp:119).
fn cartan_matrix(letter: char, rank: usize) -> Vec<Vec<i32>> {
    let mut matrix = vec![vec![0; rank]; rank];
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 2;
    }
    let link = |matrix: &mut Vec<Vec<i32>>, a: usize, b: usize, upper: i32, lower: i32| {
        let (low, high) = if a < b { (a, b) } else { (b, a) };
        matrix[low][high] = upper;
        matrix[high][low] = lower;
    };
    match letter {
        'A' => {
            for i in 0..rank.saturating_sub(1) {
                link(&mut matrix, i, i + 1, -1, -1);
            }
        }
        'B' => {
            for i in 0..rank - 1 {
                if i == rank - 2 {
                    link(&mut matrix, i, i + 1, -2, -1);
                } else {
                    link(&mut matrix, i, i + 1, -1, -1);
                }
            }
        }
        'C' => {
            for i in 0..rank - 1 {
                if i == rank - 2 {
                    link(&mut matrix, i, i + 1, -1, -2);
                } else {
                    link(&mut matrix, i, i + 1, -1, -1);
                }
            }
        }
        'D' => {
            for i in 0..rank - 3 {
                link(&mut matrix, i, i + 1, -1, -1);
            }
            link(&mut matrix, rank - 3, rank - 2, -1, -1);
            link(&mut matrix, rank - 3, rank - 1, -1, -1);
        }
        'G' => {
            link(&mut matrix, 0, 1, -1, -3);
        }
        'F' => {
            link(&mut matrix, 0, 1, -1, -1);
            link(&mut matrix, 1, 2, -2, -1);
            link(&mut matrix, 2, 3, -1, -1);
        }
        'E' => {
            // Upstream: node 0 links to node 2; node 1 to node 3; chain on.
            link(&mut matrix, 0, 2, -1, -1);
            for i in 1..rank - 1 {
                if i == 1 {
                    link(&mut matrix, 1, 3, -1, -1);
                } else {
                    link(&mut matrix, i, i + 1, -1, -1);
                }
            }
        }
        other => panic!("unsupported letter {other}"),
    }
    matrix
}

fn weyl_order(letter: char, rank: usize) -> usize {
    let factorial = |n: usize| (1..=n).product::<usize>();
    match letter {
        'A' => factorial(rank + 1),
        'B' | 'C' => (1 << rank) * factorial(rank),
        'D' => (1 << (rank - 1)) * factorial(rank),
        'G' => 12,
        'F' => 1152,
        'E' => match rank {
            6 => 51_840,
            7 => 2_903_040,
            _ => panic!("unsupported E rank"),
        },
        _ => panic!("unsupported letter"),
    }
}

fn root_count(letter: char, rank: usize) -> usize {
    match letter {
        'A' => rank * (rank + 1),
        'B' | 'C' => 2 * rank * rank,
        'D' => 2 * rank * (rank - 1),
        'G' => 12,
        'F' => 48,
        'E' => match rank {
            6 => 72,
            7 => 126,
            _ => panic!("unsupported E rank"),
        },
        _ => panic!("unsupported letter"),
    }
}

fn integer_budget() -> IntegerLatticeBudget {
    IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256)
}

fn run_group(name: &str) {
    let letter = name.chars().next().expect("letter");
    let rank: usize = name[1..].parse().expect("rank");
    let cartan = cartan_matrix(letter, rank);

    let start = Instant::now();
    // Simply connected: weight-lattice basis — roots are Cartan rows,
    // coroots the standard basis.
    let roots: Vec<Weight> = cartan.iter().cloned().map(Weight::new).collect();
    let coroots: Vec<Coweight> = (0..rank)
        .map(|index| {
            let mut coordinates = vec![0; rank];
            coordinates[index] = 1;
            Coweight::new(coordinates)
        })
        .collect();
    let datum = BasedRootDatum::from_simple_data(rank, cartan, roots, coroots).expect("datum");
    let distinguished = LatticeInvolution::identity(&datum).expect("identity");
    let inner_class =
        InnerClass::new(datum, distinguished, root_count(letter, rank)).expect("inner class");
    let class_budget = CartanClassificationBudget::new(
        integer_budget(),
        AdjointFiberBudget::new(integer_budget(), 1_000_000, 10_000_000),
        weyl_order(letter, rank),
        4_096,
        4_096,
    );
    let classification =
        CartanClassification::build(&inner_class, &class_budget).expect("classification");
    let strong = StrongRealClassification::build(&classification, 1 << 20).expect("strong");
    let order = ExternalFormOrder::build(&inner_class, &classification).expect("external order");
    let pipeline_ms = start.elapsed().as_millis();

    let kgb_start = Instant::now();
    let mut table = InvolutionTable::new(
        &inner_class,
        InvolutionTableBudget::new(1 << 20, integer_budget()),
    )
    .expect("table");
    let fundamental = classification.cartan_ids().next().expect("fundamental id");
    table
        .add_cartan(&classification, fundamental)
        .expect("fundamental cartan");
    for external in 0..order.form_count() {
        let internal = order.internal(external).expect("internal form");
        let seed = RealFormSeed::build(
            &inner_class,
            &classification,
            &strong,
            &table,
            internal,
            &integer_budget(),
            1 << 20,
        )
        .expect("seed");
        let graph = KgbGraph::build(&inner_class, &classification, &strong, &mut table, &seed)
            .expect("kgb");
        let mut lengths: Vec<usize> = graph
            .ids()
            .map(|id| graph.length(id).expect("length"))
            .collect();
        lengths.sort_unstable();
        let rendered: Vec<String> = lengths.iter().map(|value| value.to_string()).collect();
        println!(
            "{name} form {external} size {} lengths {}",
            graph.size(),
            rendered.join(",")
        );
    }
    let kgb_ms = kgb_start.elapsed().as_millis();
    println!("{name} time_ms {pipeline_ms} kgb_ms {kgb_ms}");
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let battery: Vec<String> = if arguments.is_empty() {
        [
            "A1", "A2", "A3", "A4", "B2", "B3", "B4", "C3", "C4", "D4", "G2", "F4",
        ]
        .iter()
        .map(|name| name.to_string())
        .collect()
    } else {
        arguments
    };
    for name in &battery {
        run_group(name);
    }
}
