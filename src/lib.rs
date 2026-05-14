#![feature(yield_expr)]
#![feature(gen_blocks)]

mod parse;
#[cfg(test)]
mod test;
mod world;

use std::rc::Rc;
use world::{Node, World};

#[derive(Clone)]
pub struct RedGex {
    nodes: Rc<[Node]>,
    groups: usize,
    time_same: usize,
}

impl RedGex {
    pub fn new(pattern: &str) -> Self {
        let mut nodes = Vec::new();
        let mut groups = Vec::new();
        let mut sames = 0;

        let mut res = Vec::new();
        World::sub_construct(
            format!("({pattern})").as_str(),
            &mut nodes,
            &mut groups,
            &mut sames,
            vec![],
            &mut |_, elem| res.push(elem),
            true,
        );
        assert_eq!(res[0], 0);
        Self {
            nodes: Rc::from(nodes),
            groups: groups.len(),
            time_same: sames,
        }
    }

    pub fn first_match<'a>(&self, text: &'a str) -> Option<Vec<&'a str>> {
        World::all_matchs(self, text).next()
    }

    pub fn all_matchs_vec<'a>(&self, text: &'a str) -> Vec<Vec<&'a str>> {
        World::all_matchs(self, text).collect()
    }

    pub fn all_matchs<'a>(&self, text: &'a str) -> impl Iterator<Item = Vec<&'a str>> {
        World::all_matchs(self, text)
    }
}

impl From<&str> for RedGex {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
