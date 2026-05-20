#![allow(clippy::print_stdout, clippy::print_stderr)]
//! M3 demo: walk three canonical recurrences through the master theorem,
//! print Binet's closed-form bracket count, show the amortized cost of
//! a million `Vec::push` operations, and demonstrate the three course
//! correctness translations (Optional → Option, try/except → Result,
//! mutable default arg → ownership).

use m2_empirical::ComplexityClass;
use m3_structural::{
    amortized_vec_push_cost, book_fight, brackets_closed_form, classify, contract_marker,
    find_opponent, schedule_safe, BookingError,
};

fn main() {
    println!("M3 · Structural complexity proofs + correctness translations (UFC/BJJ edition)\n");

    println!("Master theorem case classification:");
    let cases = [
        (
            "tournament bracket: 2 T(n/2) + n",
            2.0,
            2.0,
            ComplexityClass::Linear,
        ),
        (
            "binary search:      1 T(n/2) + 1",
            1.0,
            2.0,
            ComplexityClass::Constant,
        ),
        (
            "Strassen-ish:       8 T(n/2) + n^2",
            8.0,
            2.0,
            ComplexityClass::Quadratic,
        ),
    ];
    for (label, a, b, fc) in cases {
        let case = classify(a, b, fc);
        println!("  {label:<34} -> {case:?}");
    }

    println!("\nBinet's closed form (bracket count):");
    for n in [0_u32, 1, 20] {
        println!("  brackets({n:>2}) = {:>10.0}", brackets_closed_form(n));
    }

    println!("\nAmortized Vec::push cost (banker's method):");
    let a = amortized_vec_push_cost(1_000_000);
    println!(
        "  total = {:>10.0}, amortized/op = {:.3}",
        a.total, a.amortized_per_op,
    );

    println!("\nOptional -> Option:");
    let roster = ["Khabib", "McGregor"];
    let opp = find_opponent(&roster, "Khabib");
    println!("  find_opponent(roster, 'Khabib') = {opp:?}");
    let empty: [&str; 0] = [];
    let none = find_opponent(&empty, "anyone");
    println!("  find_opponent([], 'anyone')     = {none:?} (exhaustively handled)");

    println!("\ntry/except -> Result:");
    let ok = book_fight(&roster, "Khabib", "McGregor");
    println!("  book_fight(['Khabib', 'McGregor'], 'Khabib', 'McGregor') = {ok:?}");
    let err = book_fight(&["Khabib"], "Khabib", "Khabib");
    let err_label: &str = match &err {
        Err(BookingError::CannotFightSelf) => "Err(CannotFightSelf)",
        Err(BookingError::NotOnRoster(_)) => "Err(NotOnRoster)",
        Ok(_) => "Ok(_)",
    };
    println!("  book_fight(['Khabib'], 'Khabib', 'Khabib')               = {err_label}");

    println!("\nmutable default -> ownership:");
    let fresh = schedule_safe("Khabib vs McGregor", None);
    println!("  schedule_safe('Khabib vs McGregor', None)               = {fresh:?}");
    let extended = schedule_safe("Adesanya vs Pereira", Some(vec!["existing".to_owned()]));
    println!("  schedule_safe('Adesanya vs Pereira', Some(vec![\"existing\"])) = {extended:?}");

    eprintln!("{}", contract_marker());
}
