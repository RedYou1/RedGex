Cheatsheet: https://regexr.com/

all matches are structured as a Vec of all group value
``` Rust
let redgex = RedGex::new(pattern: &str);

// Some([gr0, gr1, gr2, ...])
let first: Option<Vec<&str>> = redgex.first_match(text: &str);

// [
//   // possibility 0
//   [gr0, gr1, gr2, ...],
//   // possibility 1
//   [gr0, gr1, gr2, ...],
//   ....
// ]
let matchs: impl Iterator<Item = Vec<&str>> = redgex.all_matchs(text: &str);
let collected_matchs: Vec<Vec<&str>> = redgex.all_matchs_vec(text: &str);
```