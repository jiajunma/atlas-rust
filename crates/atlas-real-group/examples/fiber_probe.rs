//! Scratch probe for the B2 block_sizes fiber undercount (oracle 4/5/12 vs
//! Rust 3/3/8). Prints, per Cartan class of the B2 simply-connected complex
//! inner class and its dual: twisted-involution orbit count, adjoint fiber
//! dimension, and per-partition-class (size, real-form label). Not part of
//! the gate; run with `cargo run -p atlas-real-group --example fiber_probe`.

use atlas_real_group::{
    dual_cartan_correspondence, dual_inner_class, AdjointFiberBudget, BasedRootDatum,
    CartanClassification, CartanClassificationBudget, InnerClass, IntegerLatticeBudget,
    LatticeInvolution, WeakRealFormId,
};

fn dump(
    side: &str,
    inner: &InnerClass,
    budget: &CartanClassificationBudget,
) -> CartanClassification {
    let classification = CartanClassification::build(inner, budget).expect("classification");
    println!(
        "== {side}: {} Cartan classes",
        classification.cartan_classes().len()
    );
    for (number, id) in classification.cartan_ids().enumerate() {
        let cartan = classification.cartan_class(id).expect("in range");
        let dimension = cartan.grading().adjoint_fiber().dimension();
        println!(
            "  cartan {number}: involution {:?}",
            cartan
                .representative()
                .root_involution()
                .involution()
                .weight_matrix()
        );
        let element_count = 1_u64 << dimension;
        let classes: Vec<WeakRealFormId> = cartan.partition().classes().collect();
        let mut sizes = vec![0_u64; classes.len()];
        for mask in 0..element_count {
            let local = cartan
                .partition()
                .class_of_mask(mask)
                .expect("mask in range");
            let index = classes.iter().position(|c| *c == local).expect("class");
            sizes[index] += 1;
        }
        println!(
            "  cartan {number}: orbits={} adjoint_dim={} classes={}",
            cartan.twisted_involution_count(),
            dimension,
            sizes.len()
        );
        for (local, (class, size)) in classes.iter().zip(sizes.iter()).enumerate() {
            let label = cartan.labels().label(*class);
            println!("    class {local} ({class:?}): size={size} label={label:?}");
        }
    }
    classification
}

fn main() {
    let integer_budget = IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256);
    let budget = CartanClassificationBudget::new(
        integer_budget.clone(),
        AdjointFiberBudget::new(integer_budget, 1_000_000, 10_000_000),
        1 << 12,
        4_096,
        4_096,
    );
    // Simply-connected B2, matching the block_sizes fixture
    // (`simply_connected(Lie_type("B2"),true)`): X = P in the fundamental
    // weight basis, so simple roots are the Cartan rows; X_* = Q^vee in the
    // simple coroot basis, so simple coroots are the standard basis.
    let cartan_matrix = vec![vec![2, -2], vec![-1, 2]];
    let datum = BasedRootDatum::from_simple_data(
        2,
        cartan_matrix.clone(),
        cartan_matrix
            .iter()
            .map(|row| atlas_real_group::Weight::new(row.clone()))
            .collect(),
        vec![
            atlas_real_group::Coweight::new(vec![1, 0]),
            atlas_real_group::Coweight::new(vec![0, 1]),
        ],
    )
    .expect("B2 sc");
    let involution = LatticeInvolution::identity(&datum).expect("identity");
    let inner = InnerClass::new(datum, involution, 1 << 12).expect("inner class");
    let classification = dump("primal B2", &inner, &budget);

    let dual = dual_inner_class(&inner, 1 << 12, 1 << 12).expect("dual inner class");
    let dual_classification = dump("dual B2", &dual, &budget);

    let correspondence = dual_cartan_correspondence(
        &inner,
        &classification,
        &dual,
        &dual_classification,
        1 << 12,
    )
    .expect("correspondence");
    for (number, (dual_id, _)) in correspondence.iter().enumerate() {
        println!("correspondence: cartan {number} <-> dual {dual_id:?}");
    }
    println!(
        "primal forms={} dual forms={}",
        classification.weak_real_form_count(),
        dual_classification.weak_real_form_count()
    );
    let fundamental = classification
        .cartan_class(classification.cartan_ids().next().expect("fundamental"))
        .expect("fundamental");
    for form in fundamental.partition().classes() {
        let set = classification.cartan_set(form).expect("form in range");
        println!("primal form {form:?} cartans {set:?}");
    }
    let dual_fundamental = dual_classification
        .cartan_class(
            dual_classification
                .cartan_ids()
                .next()
                .expect("fundamental"),
        )
        .expect("fundamental");
    for form in dual_fundamental.partition().classes() {
        let set = dual_classification.cartan_set(form).expect("form in range");
        println!("dual form {form:?} cartans {set:?}");
    }

    // Oracle-style fiberSize: strong-real full-fiber orbit sizes
    // (InnerClass::fiberSize, innerclass.cpp:603-614).
    let primal_strong =
        atlas_real_group::StrongRealClassification::build(&classification, 10_000_000)
            .expect("primal strong");
    let dual_strong =
        atlas_real_group::StrongRealClassification::build(&dual_classification, 10_000_000)
            .expect("dual strong");

    // Per-Cartan per-square-class orbit sizes, to compare with the oracle's
    // print_strong_real dump.
    let dump_strong = |side: &str,
                       strong: &atlas_real_group::StrongRealClassification,
                       classification: &CartanClassification| {
        for (number, cartan_id) in classification.cartan_ids().enumerate() {
            let data = strong.strong_real_data(cartan_id).expect("strong data");
            let cartan = classification.cartan_class(cartan_id).expect("cartan");
            print!("{side} cartan {number}:");
            for square in data.square_classes() {
                let count = data.fiber_orbit_count(square).expect("square");
                let mut sizes = Vec::new();
                for orbit in 0..count {
                    let elements = data.orbit_elements(square, orbit).expect("orbit");
                    let weak = data.weak_real_of_orbit(square, orbit).expect("weak");
                    sizes.push(format!("{}(weak {:?})", elements.len(), weak));
                }
                print!(" class {}: [{}]", square.index(), sizes.join(", "));
            }
            println!();
            let _ = cartan;
        }
    };
    dump_strong("primal", &primal_strong, &classification);
    dump_strong("dual", &dual_strong, &dual_classification);

    let fiber_size = |strong: &atlas_real_group::StrongRealClassification,
                      classification: &CartanClassification,
                      cartan_id: atlas_real_group::CartanId,
                      form: WeakRealFormId| {
        let cartan = classification.cartan_class(cartan_id).expect("cartan");
        let data = strong.strong_real_data(cartan_id).expect("strong data");
        let local = cartan
            .partition()
            .classes()
            .find(|class| cartan.labels().label(*class) == Some(form))
            .expect("form occurs at this cartan");
        let rep = data.strong_real_form(local).expect("strong rep");
        data.orbit_elements(rep.square_class(), rep.fiber_orbit())
            .expect("orbit")
            .len() as u64
    };

    let forms: Vec<WeakRealFormId> = fundamental.partition().classes().collect();
    let dual_forms: Vec<WeakRealFormId> = dual_fundamental.partition().classes().collect();
    for &form in &forms {
        for &dual_form in &dual_forms {
            let cartan_set = classification.cartan_set(form).expect("form");
            let dual_set = dual_classification
                .cartan_set(dual_form)
                .expect("dual form");
            let mut total = 0_u64;
            for (number, cartan_id) in classification.cartan_ids().enumerate() {
                if !cartan_set.contains(&cartan_id) {
                    continue;
                }
                let (dual_id, _) = &correspondence[number];
                if !dual_set.contains(dual_id) {
                    continue;
                }
                let cartan = classification.cartan_class(cartan_id).expect("cartan");
                let orbit = cartan.twisted_involution_count() as u64;
                let fs = fiber_size(&primal_strong, &classification, cartan_id, form);
                let dfs = fiber_size(&dual_strong, &dual_classification, *dual_id, dual_form);
                println!(
                    "  ({form:?},{dual_form:?}) cartan {number}: orbit={orbit} fs={fs} dfs={dfs} -> {}",
                    orbit * fs * dfs
                );
                total += orbit * fs * dfs;
            }
            println!("matrix ({form:?},{dual_form:?}) = {total}");
        }
    }
    // Oracle prints forms most-split-first: expect
    // | 0, 0,  1 | / | 0, 0,  4 | / | 1, 5, 12 |
}
