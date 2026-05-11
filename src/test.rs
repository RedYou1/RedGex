use std::rc::Rc;

use crate::*;

#[test]
fn or_in_root1() {
    assert_eq!(
        Vec::<usize>::new(),
        parse::or_in_root("(awed|dwaevsr)[awdef|adefsrvr]a")
    );
    assert_eq!(
        Vec::<usize>::new(),
        parse::or_in_root("(awed|dwaevsr)[awdef|adefsrvr]\\|a")
    );
    assert_eq!(
        vec![30],
        parse::or_in_root("(awed|dwaevsr)[awdef|adefsrvr]|a")
    );
    assert_eq!(
        vec![14, 31],
        parse::or_in_root("(awed|dwaevsr)|[awdef|adefsrvr]|a")
    );
}

#[test]
fn get_end_of_group1() {
    assert_eq!(None, parse::get_end_of_group(""));
    assert_eq!(None, parse::get_end_of_group("("));
    assert_eq!(None, parse::get_end_of_group("(\\)"));
    assert_eq!(Some(1), parse::get_end_of_group("()"));
    assert_eq!(Some(2), parse::get_end_of_group("(a)"));
    assert_eq!(Some(2), parse::get_end_of_group("(|)"));
}

#[test]
fn get_end_of_any1() {
    assert_eq!(None, parse::get_end_of_any(""));
    assert_eq!(None, parse::get_end_of_any("["));
    assert_eq!(Some(1), parse::get_end_of_any("[]"));
    assert_eq!(Some(2), parse::get_end_of_any("[a]"));
    assert_eq!(Some(2), parse::get_end_of_any("[|]"));
}

#[test]
fn expand_any1() {
    assert_eq!(Rc::from(""), parse::expand_any(""));
    assert_eq!(Rc::from("a"), parse::expand_any("a"));
    assert_eq!(Rc::from(parse::PREFAB[0].1), parse::expand_any("\\w"));
    assert_eq!(Rc::from("a|bghijkl"), parse::expand_any("a|bg-l"));

    assert_eq!(Rc::from(""), parse::expand_any(""));
    assert_eq!(Rc::from("a"), parse::expand_any("a"));
    assert_eq!(Rc::from(parse::PREFAB[0].1), parse::expand_any("\\w"));
    assert_eq!(Rc::from("a|bghijkl"), parse::expand_any("a|bg-l"));
}

#[test]
#[should_panic]
fn expand_any2() {
    assert_eq!(Rc::from(parse::PREFAB[0].1), parse::expand_any("\\W"));
}

#[test]
fn get_end_of_static1() {
    assert_eq!(1, parse::get_end_of_static("l"));
    assert_eq!(5, parse::get_end_of_static("lawdf"));
    assert_eq!(5, parse::get_end_of_static("lawdf(defsvrfb)"));
    assert_eq!(5, parse::get_end_of_static("lawdf?"));
    assert_eq!(5, parse::get_end_of_static("lawdf+"));
    assert_eq!(5, parse::get_end_of_static("lawdf[ae]"));
}

#[test]
fn matchs1() {
    assert_eq!(
        vec![
            vec!["A"],
            vec!["B"],
            vec!["C"],
            vec!["AB"],
            vec!["BC"],
            vec!["ABC"]
        ],
        crate::matchs("ABC", "[a-zA-Z]+")
    );
    assert_eq!(
        vec![
            vec![""],
            vec!["A"],
            vec!["B"],
            vec!["C"],
            vec!["AB"],
            vec!["BC"],
            vec!["ABC"]
        ],
        crate::matchs("ABC", "[a-zA-Z]*")
    );
    assert_eq!(
        vec![vec![""], vec!["A"], vec!["AB"], vec!["ABC"]],
        crate::matchs("ABC", "^[a-zA-Z]*")
    );
    assert_eq!(
        vec![vec!["C"], vec!["BC"], vec!["ABC"]],
        crate::matchs("ABC", "[a-zA-Z]*$")
    );
    assert_eq!(vec![vec!["ABC"]], crate::matchs("ABC", "^[a-zA-Z]*$"));
}

#[test]
fn matchs2() {
    assert_eq!(
        vec![vec!["abbabacabbaba", "abbaba", "a"]],
        crate::matchs("abbabacabbaba", "^((a|b)*)c\\1$")
    );
}

#[test]
fn matchs3() {
    let pattern = "^([\\w;]{,5})\\D\\s(\\$|3|8|\\1)$";

    assert_eq!(
        vec![vec!["a7;r,\t$", "a7;r", "$"]],
        crate::matchs("a7;r,\t$", pattern)
    );

    assert_eq!(
        vec![vec!["a7;r,\ta7;r", "a7;r", "a7;r"]],
        crate::matchs("a7;r,\ta7;r", pattern)
    );
}

#[test]
fn match4() {
    assert!(
        crate::matchs(
            "Jan 22 09:15:05 127.0.0.1 [S=207470958] [SID=90ae9b:28:6748965]  (N 13929029)  (#3125)gwSession[Allocated]. Handle:0000005576E58A90; Global session ID: 5ad3083ec1e86aee [Time:20-01@09:15:04.832]",
            "([A-z]{3} [\\d]{2} [\\d]{1,2}:[\\d]{1,2}:[\\d]{1,2}) ([\\d]{1,3}\\.[\\d]{1,3}\\.[\\d]{1,3}\\.[\\d]{1,3}) (\\[S\\=[\\d]{9}\\]) (\\[[A-z]ID=.{1,18}\\])\\s{1,3}(\\(N\\s[\\d]{5,20}\\))?(\\s+(.*))\\s{1,3}(\\[Time:.*\\])?"
        ).iter().any(|v| v[0] == "Jan 22 09:15:05 127.0.0.1 [S=207470958] [SID=90ae9b:28:6748965]  (N 13929029)  (#3125)gwSession[Allocated]. Handle:0000005576E58A90; Global session ID: 5ad3083ec1e86aee [Time:20-01@09:15:04.832]")
    );
}
