use crate::world::World;

mod parse;
#[cfg(test)]
mod test;
mod world;

pub fn matchs<'a>(text: &'a str, pattern: &str) -> Vec<Vec<&'a str>> {
    World::matchs(text, pattern)
}
