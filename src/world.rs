use std::{
    collections::{HashSet, VecDeque},
    ops::{Range, RangeInclusive},
    rc::Rc,
};

use crate::parse::{
    PREFAB, expand_any, get_end_of_any, get_end_of_group, get_end_of_static, or_in_root,
};

#[derive(Debug)]
pub enum Check {
    Start(),
    End(bool),

    StartGroup(usize),
    EndGroup(usize),

    Static(Rc<str>),
    AnyOf(Rc<str>),
    NoneOf(Rc<str>),
    MultipleOf(usize, usize, RangeInclusive<usize>, bool),
    SameAs(usize),
}

#[derive(Debug)]
pub struct Node {
    check: Check,
    nexts: Vec<usize>,
}

impl Node {
    pub fn create(
        check: Check,
        pattern: &str,
        nodes: &mut Vec<Node>,
        groups: &mut Vec<usize>,
        sames: &mut usize,
        mut parents: Vec<usize>,
        push_end: bool,
    ) -> usize {
        let id = nodes.len();
        nodes.push(Node {
            check,
            nexts: Vec::new(),
        });
        parents.push(id);
        World::sub_construct(
            pattern,
            nodes,
            groups,
            sames,
            parents,
            &mut |nodes, elem| unsafe { nodes.as_mut_unchecked()[id].nexts.insert(0, elem) },
            push_end,
        );
        id
    }

    pub fn create_dont_append_parent(
        check: Check,
        pattern: &str,
        nodes: &mut Vec<Node>,
        groups: &mut Vec<usize>,
        sames: &mut usize,
        parents: Vec<usize>,
        push_end: bool,
    ) -> usize {
        let id = nodes.len();
        nodes.push(Node {
            check,
            nexts: Vec::new(),
        });
        World::sub_construct(
            pattern,
            nodes,
            groups,
            sames,
            parents,
            &mut |nodes, elem| unsafe { nodes.as_mut_unchecked()[id].nexts.insert(0, elem) },
            push_end,
        );
        id
    }
}

#[derive(Debug, Clone)]
pub struct World {
    current_idx: usize,

    groups: Vec<Option<Range<usize>>>,
    register_group: Vec<Option<usize>>,
    nodes: Rc<[Node]>,
    current_node: Option<usize>,
    time_same: Vec<usize>,
}

impl World {
    pub fn next(mut self, text: &str) -> Option<Vec<World>> {
        let current_node = &self.nodes[self.current_node.unwrap()];
        if !match &current_node.check {
            Check::Start() => self.current_idx == 0,
            Check::End(end) => !end || self.current_idx == text.len(),
            Check::StartGroup(_) => true,
            Check::EndGroup(id) => self.register_group[*id].is_some(),
            Check::Static(check) => {
                self.current_idx < text.len()
                    && text[self.current_idx..].starts_with(check.as_ref())
            }
            Check::AnyOf(poss) => {
                self.current_idx < text.len()
                    && poss.contains(&text[self.current_idx..=self.current_idx])
            }
            Check::NoneOf(poss) => {
                self.current_idx < text.len()
                    && !poss.contains(&text[self.current_idx..=self.current_idx])
            }
            Check::MultipleOf(_, _, _, _) => true, //later
            Check::SameAs(id) => {
                if let Some(r) = &self.groups[*id] {
                    self.current_idx < text.len()
                        && text[self.current_idx..].starts_with(&text[r.clone()])
                } else {
                    false
                }
            }
        } {
            return None;
        }
        let current_idx = self.current_idx
            + match &current_node.check {
                Check::Start() | Check::End(_) => 0,
                Check::StartGroup(id) => {
                    self.register_group[*id] = Some(self.current_idx);
                    0
                }
                Check::EndGroup(id) => {
                    self.groups[*id] = Some(self.register_group[*id].unwrap()..self.current_idx);
                    0
                }
                Check::Static(text) => text.len(),
                Check::AnyOf(_) | Check::NoneOf(_) => 1,
                Check::MultipleOf(_, same_id, _, _) => {
                    self.time_same[*same_id] += 1;
                    0
                }
                Check::SameAs(id) => self.groups[*id].as_ref().unwrap().len(),
            };
        let mut nexts = current_node.nexts.clone();
        if let Check::MultipleOf(id, same_id, range_inclusive, big_first) = &current_node.check {
            let time = self.time_same[*same_id];
            if time < *range_inclusive.start() {
                nexts = vec![*id];
            } else if range_inclusive.contains(&time) {
                nexts.push(*id);
                if *big_first {
                    nexts.reverse();
                }
            } else {
                return None;
            }
        }
        Some(
            nexts
                .iter()
                .map(|&node| {
                    let mut w = self.clone();
                    w.current_node = Some(node);
                    w.current_idx = current_idx;
                    w
                })
                .collect(),
        )
    }

    fn leafs(nodes: &Vec<Node>, of: usize) -> Vec<usize> {
        let current = &nodes[of];
        if current.nexts.is_empty() {
            vec![of]
        } else {
            let mut r = Vec::new();
            for t in current
                .nexts
                .iter()
                .flat_map(|next| World::leafs(nodes, *next))
            {
                if !r.contains(&t) {
                    r.push(t);
                }
            }
            r
        }
    }

    fn sub_construct(
        pattern: &str,
        nodes: &mut Vec<Node>,
        groups: &mut Vec<usize>,
        sames: &mut usize,
        mut parents: Vec<usize>,
        parent_push: &mut impl FnMut(*mut Vec<Node>, usize),
        push_end: bool,
    ) {
        if pattern.is_empty() {
            if push_end {
                let id = nodes.len();
                nodes.push(Node {
                    check: Check::End(false),
                    nexts: Vec::new(),
                });
                parent_push(nodes, id);
            }
            return;
        }
        let ors = or_in_root(pattern);
        if !ors.is_empty() {
            World::sub_construct(
                &pattern[..*ors.first().unwrap()],
                nodes,
                groups,
                sames,
                parents.clone(),
                parent_push,
                push_end,
            );
            for window in ors.windows(2) {
                World::sub_construct(
                    &pattern[(window[0] + 1)..window[1]],
                    nodes,
                    groups,
                    sames,
                    parents.clone(),
                    parent_push,
                    push_end,
                );
            }
            World::sub_construct(
                &pattern[(*ors.last().unwrap() + 1)..],
                nodes,
                groups,
                sames,
                parents,
                parent_push,
                push_end,
            );
            return;
        }

        match &pattern.chars().collect::<Vec<char>>()[..] {
            ['(', '?', ..] => panic!("none capturing group not supported."),
            ['(', ..] => {
                let end =
                    get_end_of_group(pattern).expect("pattern not valide. end of group not found.");
                let gid = groups.len();
                groups.push(usize::MAX); //allocate the location
                let id = Node::create(
                    Check::StartGroup(gid),
                    &pattern[1..end],
                    nodes,
                    groups,
                    sames,
                    vec![],
                    false,
                );
                groups[gid] = id;

                parents.push(id);
                let end_id = Node::create_dont_append_parent(
                    Check::EndGroup(gid),
                    &pattern[(end + 1)..],
                    nodes,
                    groups,
                    sames,
                    parents,
                    push_end,
                );
                for next in World::leafs(nodes, id) {
                    nodes[next].nexts.push(end_id);
                }
                parent_push(nodes, id);
            }
            ['[', '^', ..] => {
                let end =
                    get_end_of_any(pattern).expect("pattern not valide. end of any not found.");

                parent_push(
                    nodes,
                    Node::create(
                        Check::NoneOf(expand_any(&pattern[2..end])),
                        &pattern[(end + 1)..],
                        nodes,
                        groups,
                        sames,
                        parents,
                        push_end,
                    ),
                );
            }
            ['[', ..] => {
                let end =
                    get_end_of_any(pattern).expect("pattern not valide. end of any not found.");

                parent_push(
                    nodes,
                    Node::create(
                        Check::AnyOf(expand_any(&pattern[1..end])),
                        &pattern[(end + 1)..],
                        nodes,
                        groups,
                        sames,
                        parents,
                        push_end,
                    ),
                )
            }
            ['.', ..] => parent_push(
                nodes,
                Node::create(
                    Check::NoneOf(Rc::from("")),
                    &pattern[1..],
                    nodes,
                    groups,
                    sames,
                    parents,
                    push_end,
                ),
            ),
            ['?', ..] => {
                if let Check::MultipleOf(_, _, _, big_first) =
                    &mut nodes[*parents.last().unwrap()].check
                {
                    assert!(*big_first);
                    *big_first = false;
                    World::sub_construct(
                        &pattern[1..],
                        nodes,
                        groups,
                        sames,
                        parents,
                        parent_push,
                        push_end,
                    );
                } else {
                    let ggp = *parents.iter().nth_back(1).unwrap();
                    let same = *sames;
                    *sames += 1;
                    let r = Node::create(
                        Check::MultipleOf(*parents.last().unwrap(), same, 0..=1, true),
                        &pattern[1..],
                        nodes,
                        groups,
                        sames,
                        parents,
                        push_end,
                    );
                    nodes[ggp].nexts.push(r);
                    parent_push(nodes, r)
                }
            }
            ['*', ..] => {
                let ggp = *parents.iter().nth_back(1).unwrap();
                let same = *sames;
                *sames += 1;
                let r = Node::create(
                    Check::MultipleOf(*parents.last().unwrap(), same, 0..=usize::MAX, true),
                    &pattern[1..],
                    nodes,
                    groups,
                    sames,
                    parents,
                    push_end,
                );
                nodes[ggp].nexts.push(r);
                parent_push(nodes, r)
            }
            ['+', ..] => {
                let same = *sames;
                *sames += 1;
                let r = Node::create(
                    Check::MultipleOf(*parents.last().unwrap(), same, 1..=usize::MAX, true),
                    &pattern[1..],
                    nodes,
                    groups,
                    sames,
                    parents,
                    push_end,
                );
                parent_push(nodes, r)
            }
            ['^', ..] => parent_push(
                nodes,
                Node::create(
                    Check::Start(),
                    &pattern[1..],
                    nodes,
                    groups,
                    sames,
                    parents,
                    push_end,
                ),
            ),
            ['$', ..] => parent_push(
                nodes,
                Node::create(
                    Check::End(true),
                    &pattern[1..],
                    nodes,
                    groups,
                    sames,
                    parents,
                    push_end,
                ),
            ),
            ['\\', c, ..] if !".*+?(){}[]|^$\\".contains(*c) => {
                if let Some(s) = PREFAB
                    .iter()
                    .find_map(|&(p, s)| p.eq_ignore_ascii_case(c).then_some(s))
                {
                    parent_push(
                        nodes,
                        Node::create(
                            if c.to_ascii_lowercase() == *c {
                                Check::AnyOf(Rc::from(s))
                            } else {
                                Check::NoneOf(Rc::from(s))
                            },
                            &pattern[2..],
                            nodes,
                            groups,
                            sames,
                            parents,
                            push_end,
                        ),
                    )
                } else if c.is_numeric() {
                    let mut group = c.to_string();
                    let mut pattern = &pattern[2..];
                    while let Some(c) = pattern.chars().next()
                        && c.is_numeric()
                    {
                        group.push(c);
                        pattern = &pattern[1..];
                    }
                    *sames += 1;
                    parent_push(
                        nodes,
                        Node::create(
                            Check::SameAs(group.parse().unwrap()),
                            pattern,
                            nodes,
                            groups,
                            sames,
                            parents,
                            push_end,
                        ),
                    )
                } else {
                    panic!("escape '{}' not supported.", *c);
                }
            }
            ['{', ..] => {
                let end = pattern.find("}").unwrap();
                let inner = &pattern[1..end];
                if inner.contains(',') {
                    let &[a, b] = &inner.split(',').collect::<Vec<&str>>()[..] else {
                        panic!("MultipleOf {{}} format error");
                    };
                    let min = if a.is_empty() { 0 } else { a.parse().unwrap() };
                    let max = if b.is_empty() {
                        usize::MAX
                    } else {
                        b.parse().unwrap()
                    };
                    let ggp = if min == 0 {
                        *parents.iter().nth_back(1).unwrap()
                    } else {
                        0
                    };
                    let same = *sames;
                    *sames += 1;
                    let r = Node::create(
                        Check::MultipleOf(*parents.last().unwrap(), same, min..=max, true),
                        &pattern[(end + 1)..],
                        nodes,
                        groups,
                        sames,
                        parents,
                        push_end,
                    );
                    if min == 0 {
                        nodes[ggp].nexts.push(r);
                    }
                    parent_push(nodes, r)
                } else {
                    let amount = inner.parse().unwrap();
                    let same = *sames;
                    *sames += 1;
                    assert!(amount > 0);
                    parent_push(
                        nodes,
                        Node::create(
                            Check::MultipleOf(
                                *parents.last().unwrap(),
                                same,
                                amount..=amount,
                                true,
                            ),
                            &pattern[(end + 1)..],
                            nodes,
                            groups,
                            sames,
                            parents,
                            push_end,
                        ),
                    )
                }
            }
            _ => {
                let end = get_end_of_static(pattern);
                let mut escape = false;
                parent_push(
                    nodes,
                    Node::create(
                        Check::Static(Rc::from(
                            pattern[..end]
                                .chars()
                                .filter(|c| {
                                    if *c == '\\' {
                                        if escape {
                                            escape = false;
                                            true
                                        } else {
                                            escape = true;
                                            false
                                        }
                                    } else {
                                        escape = false;
                                        true
                                    }
                                })
                                .collect::<String>(),
                        )),
                        &pattern[end..],
                        nodes,
                        groups,
                        sames,
                        parents,
                        push_end,
                    ),
                )
            }
        }
    }

    pub fn construct(pattern: &str) -> World {
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
        World {
            current_idx: 0,
            groups: vec![None; groups.len()],
            register_group: vec![None; groups.len()],
            nodes: Rc::from(nodes),
            current_node: res.first().cloned(),
            time_same: vec![0; sames],
        }
    }

    pub fn first_match<'a>(text: &'a str, pattern: &str) -> Option<Vec<&'a str>> {
        let mut futures = VecDeque::with_capacity(text.len() + 10);
        {
            let w = World::construct(pattern);
            for c in 1..text.len() {
                let mut w = w.clone();
                w.current_idx = c;
                futures.push_back(w);
            }
            futures.push_front(w);
        }
        while let Some(current) = futures.pop_front() {
            if let Some(nexts) = current.next(text) {
                let mut i = 0;
                for next in nexts {
                    if let Some(node) = next.current_node {
                        if let Check::End(false) = next.nodes[node].check {
                            let r = next
                                .groups
                                .iter()
                                .map(|g| {
                                    if let Some(g) = g {
                                        &text[g.clone()]
                                    } else {
                                        &text[0..0]
                                    }
                                })
                                .collect();
                            return Some(r);
                        } else {
                            futures.insert(i, next);
                            i += 1;
                        }
                    }
                }
            }
        }
        None
    }

    pub fn all_matchs<'a>(text: &'a str, pattern: &str) -> Vec<Vec<&'a str>> {
        let mut futures = VecDeque::with_capacity(text.len() + 10);
        {
            let w = World::construct(pattern);
            for c in 1..text.len() {
                let mut w = w.clone();
                w.current_idx = c;
                futures.push_back(w);
            }
            futures.push_front(w);
        }
        let mut res = Vec::new();
        while let Some(current) = futures.pop_front() {
            if let Some(nexts) = current.next(text) {
                let mut i = 0;
                for next in nexts {
                    if let Some(node) = next.current_node {
                        if let Check::End(false) = next.nodes[node].check {
                            let r = next
                                .groups
                                .iter()
                                .map(|g| {
                                    if let Some(g) = g {
                                        &text[g.clone()]
                                    } else {
                                        &text[0..0]
                                    }
                                })
                                .collect();
                            if !res.contains(&r) {
                                res.push(r);
                            }
                        } else {
                            futures.insert(i, next);
                            i += 1;
                        }
                    }
                }
            }
        }
        res
    }
}
