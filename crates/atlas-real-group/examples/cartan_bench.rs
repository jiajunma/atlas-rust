//! Scratch profiling harness (NOT for merge): time `CartanClassification::build`
//! for the split E8 inner class in a loop, so perf record can attribute the
//! build's internal phases.
//!
//! Run: `cartan_bench [reps]` — default 10 reps.

use std::time::Instant;

use atlas_real_group::{
    AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget, Coweight,
    InnerClass, IntegerLatticeBudget, LatticeInvolution, Weight,
};

fn e8_cartan() -> Vec<Vec<i32>> {
    let rank = 8;
    let mut matrix = vec![vec![0; rank]; rank];
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 2;
    }
    let mut link = |matrix: &mut Vec<Vec<i32>>, a: usize, b: usize| {
        matrix[a][b] = -1;
        matrix[b][a] = -1;
    };
    // Upstream E8: node 0 links to node 2; node 1 to node 3; chain on.
    link(&mut matrix, 0, 2);
    link(&mut matrix, 1, 3);
    for i in 2..rank - 1 {
        link(&mut matrix, i, i + 1);
    }
    matrix
}

fn main() {
    let reps: usize = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(10);
    let cartan = e8_cartan();
    let roots: Vec<Weight> = cartan.iter().cloned().map(Weight::new).collect();
    let coroots: Vec<Coweight> = (0..8)
        .map(|index| {
            let mut coordinates = vec![0; 8];
            coordinates[index] = 1;
            Coweight::new(coordinates)
        })
        .collect();
    let datum = BasedRootDatum::from_simple_data(8, cartan, roots, coroots).expect("datum");
    let distinguished = LatticeInvolution::identity(&datum).expect("identity");
    let inner_class = InnerClass::new(datum, distinguished, 240).expect("inner class");
    let integer_budget = IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256);
    let class_budget = CartanClassificationBudget::new(
        integer_budget.clone(),
        AdjointFiberBudget::new(integer_budget, 1_000_000, 10_000_000),
        4_000_000,
        4_096,
        4_096,
    );
    for rep in 0..reps {
        let started = Instant::now();
        let classification =
            CartanClassification::build(&inner_class, &class_budget).expect("classification");
        println!(
            "rep {rep}: ms={} classes={} members={}",
            started.elapsed().as_millis(),
            classification.cartan_classes().len(),
            classification.twisted_involution_count()
        );
    }
}
