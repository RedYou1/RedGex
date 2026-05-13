use crate::world::World;

mod parse;
#[cfg(test)]
mod test;
mod world;

pub fn first_match<'a>(text: &'a str, pattern: &str) -> Option<Vec<&'a str>> {
    World::first_match(text, pattern)
}

pub fn all_matchs<'a>(text: &'a str, pattern: &str) -> Vec<Vec<&'a str>> {
    World::all_matchs(text, pattern)
}
