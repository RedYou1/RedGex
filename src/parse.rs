use std::rc::Rc;

pub const PREFAB: [(char, &str); 3] = [
    (
        'w',
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
    ),
    ('d', "0123456789"),
    ('s', " \t\n"),
];

pub fn or_in_root(text: &str) -> Vec<usize> {
    let mut inside: usize = 0;
    let mut escaped = false;
    let mut big_escaped = false;
    text.chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if c == '\\' || escaped {
                escaped = !escaped;
                None
            } else if !big_escaped && c == '(' {
                inside += 1;
                None
            } else if !big_escaped && c == ')' {
                assert_ne!(inside, 0);
                inside -= 1;
                None
            } else if c == '[' {
                big_escaped = true;
                inside += 1;
                None
            } else if c == ']' {
                big_escaped = false;
                assert_ne!(inside, 0);
                inside -= 1;
                None
            } else {
                (inside == 0 && c == '|').then_some(i)
            }
        })
        .collect()
}

pub fn get_end_of_group(text: &str) -> Option<usize> {
    let mut inside: usize = 0;
    let mut escaped = false;
    let mut big_escaped = false;
    for (i, c) in text.chars().enumerate() {
        if c == '\\' || escaped {
            escaped = !escaped;
        } else if !big_escaped && c == '(' {
            inside += 1;
        } else if !big_escaped && c == ')' {
            assert_ne!(inside, 0);
            inside -= 1;
            if inside == 0 {
                return Some(i);
            }
        } else if c == '[' {
            big_escaped = true;
            inside += 1;
        } else if c == ']' {
            big_escaped = false;
            assert_ne!(inside, 0);
            inside -= 1;
        }
    }
    None
}

pub fn get_end_of_any(text: &str) -> Option<usize> {
    let mut inside: usize = 0;
    let mut escaped = false;
    for (i, c) in text.chars().enumerate() {
        if c == '\\' || escaped {
            escaped = !escaped;
        } else if c == '[' {
            inside += 1;
        } else if c == ']' {
            assert_ne!(inside, 0);
            inside -= 1;
            if inside == 0 {
                return Some(i);
            }
        }
    }
    None
}

const ALPHA1: &str = "abcdefghijklmnopqrstuvwxyz";
const ALPHA2: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHA3: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const NUMS: &str = "0123456789";
pub fn expand_any(text: &str) -> Rc<str> {
    let mut res = String::with_capacity(text.len());
    let mut escaped = false;
    for c in text.chars() {
        if escaped {
            if let Some(p) = PREFAB
                .iter()
                .find_map(|&(p1, p2)| p1.eq_ignore_ascii_case(&c).then_some(p2))
            {
                if c.to_ascii_lowercase() != c {
                    panic!("negative wildcard in [] not supported");
                }
                res += p;
            } else {
                res.push(c);
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if let Some(last) = res.chars().last()
            && last == '-'
        {
            res.pop();
            let prev = res.pop().unwrap();
            if c.is_numeric() {
                let s = NUMS.find(prev).unwrap();
                let e = NUMS.find(c).unwrap();
                res += &NUMS[s..=e];
            } else {
                let p = if prev.to_ascii_lowercase() == prev {
                    assert!(c.to_ascii_lowercase() == c, "expand_any:{text}");
                    ALPHA1
                } else if c.to_ascii_lowercase() == c {
                    ALPHA3
                } else {
                    ALPHA2
                };
                let s = p.find(prev).unwrap();
                let e = p.find(c).unwrap();
                res += &p[s..=e];
            }
        } else {
            res.push(c);
        }
    }
    Rc::from(res.as_str())
}

pub fn get_end_of_static(text: &str) -> usize {
    let mut res = 0;
    let mut escaped = false;
    for c in text.chars() {
        if escaped {
            if PREFAB
                .iter()
                .find(|&(p, _)| c.eq_ignore_ascii_case(p))
                .is_some()
            {
                return res;
            }
            if c.is_numeric() {
                return res - 1;
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if "(){}[]|^$.*+?".contains(c) {
            return res;
        }
        res += 1;
    }
    res
}
