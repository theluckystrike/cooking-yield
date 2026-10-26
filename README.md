# cooking-yield

Small, dependency-free Rust helpers for one recurring kitchen-math problem:
food is weighed in one state (raw or cooked) and looked up in the other.

Cooking removes water (meat) or adds it (pasta, rice) but does not change the
calories in the piece, so the calorie *density* changes. This crate converts
weights through a cooking yield and keeps the calorie arithmetic consistent.

```rust
use cooking_yield::{Yield, chicken_breast_roasted};

let y = chicken_breast_roasted();          // USDA 2012 mean yield, 72 %
assert_eq!(y.cooked_from_raw(500.0).round(), 360.0);

// 150 g of cooked chicken logged against a raw entry of 120 kcal/100 g
let raw_equiv = y.raw_from_cooked(150.0);  // ~208 g raw
let kcal = raw_equiv * 120.0 / 100.0;      // ~250 kcal, not 180
assert_eq!(kcal.round(), 250.0);

// Measure your own yield
let mine = Yield::measured(480.0, 331.0).unwrap();
assert!((mine.fraction() - 0.6896).abs() < 1e-3);
```

The bundled constants are a handful of mean yields from the USDA *Table of
Cooking Yields for Meat and Poultry* (Release 2, December 2012). Real pieces
vary; measure your own yield when accuracy matters.

An interactive version with the full USDA table by cut and cooking method is
the [raw to cooked weight calculator](https://recipetool.net/tools/raw-to-cooked-weight-calculator)
on RecipeTool.

## License

MIT
