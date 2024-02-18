extern crate ansi_term;
use std::env;
use std::io;
use std::io::Write;
use std::process::exit;
use ansi_term::Colour;

#[derive(Debug, PartialEq, Clone)]
enum Expression {
    Literal(char),
    Digit,
    Alphanumeric,
    Group(String),
    NegativeGroup(String),
    Start,
    End,
    Wildcard,
    OneOrMore,
    ZeroOrMore,
    Alternations(Vec<Vec<Expression>>)
}

fn pattern_to_expressions(pattern: &str) -> Vec<Expression> {
    let mut i = 0;
    let mut expressions = Vec::new();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    while i < pattern_chars.len() {
        if pattern_chars[i] == '\\' {
            let expression = if pattern_chars[i+1] == 'd' {
                Expression::Digit
            } else if pattern_chars[i+1] == 'w' {
                Expression::Alphanumeric
            } else {
                Expression::Literal(pattern_chars[i+1].clone())
            };
            expressions.push(expression);
            i += 2;
        } else if pattern_chars[i] == '[' {
            let negative = pattern_chars[i+1] == '^';
            i += 1 + (negative as usize);
            let mut group = String::new();
            while pattern_chars[i] != ']' {
                group.push(pattern_chars[i]);
                i += 1;
            }
            i += 1;
            let expression: Expression = if negative {
                Expression::NegativeGroup(group)
            } else {
                Expression::Group(group)
            };
            expressions.push(expression);
        } else if pattern_chars[i] == '^' {
            expressions.push(Expression::Start);
            i += 1;
        } else if pattern_chars[i] == '$' {
            expressions.push(Expression::End);
            i += 1;
        } else if pattern_chars[i] == '.' {
            expressions.push(Expression::Wildcard);
            i += 1;
        } else if pattern_chars[i] == '+' {
            let last_expression = expressions.pop().unwrap();
            expressions.push(Expression::OneOrMore);
            expressions.push(last_expression);
            i += 1;
        } else if pattern_chars[i] == '*' {
            let last_expression = expressions.pop().unwrap();
            expressions.push(Expression::ZeroOrMore);
            expressions.push(last_expression);
            i += 1;
        } else if pattern_chars[i] == '(' {
            let mut alternations: Vec<Vec<Expression>> = Vec::new();
            while pattern_chars[i] != ')' {
                i += 1;
                let mut new_pattern = "".to_string();
                while pattern_chars[i] != '|' && pattern_chars[i] != ')' {
                    new_pattern.push(pattern_chars[i]);
                    i += 1;
                }
                alternations.push(pattern_to_expressions(&new_pattern));
            }
            expressions.push(Expression::Alternations(alternations));
            i += 1;
        } else {
            expressions.push(Expression::Literal(pattern_chars[i].clone()));
            i += 1;
        }
    }
    expressions
}

fn append_to_result(mut result: Vec<u8>, mut to_append: Vec<u8>) -> Vec<u8> {
    result.append(&mut to_append);
    result
}

fn match_pattern_from(input_chars: &mut std::str::Chars<'_>, pattern: &Vec<Expression>, offset: usize) -> Option<Vec<u8>> {
    let mut result:Vec<u8> = Vec::new();
    let mut new_offset = offset;
    for expression_idx in 0..pattern.len() {
        let expression = &pattern[expression_idx];
        match expression {
            Expression::Start => continue,
            Expression::OneOrMore => {
                let next_expression = &pattern[expression_idx + 1];
                let mut rest = Vec::from(&pattern[(expression_idx + 1)..]);
                rest.insert(0, Expression::ZeroOrMore);
                rest.insert(0, next_expression.clone());
                return match match_pattern_from(&mut input_chars.clone(), &mut rest, new_offset) {
                    Some(partial_result) => {
                        Some(append_to_result(result, partial_result))
                    },
                    None => None
                }
            },
            Expression::ZeroOrMore => {
                let next_expression = &pattern[expression_idx + 1];
                let mut rest_with = Vec::from(&pattern[(expression_idx + 1)..]);
                rest_with.insert(0, Expression::ZeroOrMore);
                rest_with.insert(0, next_expression.clone());
                match match_pattern_from(&mut input_chars.clone(), &mut rest_with, new_offset) {
                    Some(partial_result) => return Some(append_to_result(result, partial_result)),
                    None => {}
                }
                let mut rest_wo = Vec::from(&pattern[(expression_idx + 2)..]);
                return match match_pattern_from(&mut input_chars.clone(), &mut rest_wo, new_offset) {
                    Some(partial_result) => Some(append_to_result(result, partial_result)),
                    None => None
                }
            },
            Expression::Alternations(alternations) => {
                for alternation in alternations.into_iter() {
                    let mut rest = Vec::from(&pattern[(expression_idx + 1)..]);
                    let mut alternation_clone = alternation.clone();
                    alternation_clone.append(&mut rest);
                    match match_pattern_from(&mut input_chars.clone(), &mut alternation_clone, new_offset) {
                        Some(partial_result) => return Some(append_to_result(result, partial_result)),
                        None => {}
                    }
                }
                return None
            }
            _ => {}
        }
        let next_char = match input_chars.next() {
            Some(c) => {
                result.push(new_offset as u8);
                new_offset += 1;
                c
            },
            None => {
                if *expression == Expression::End {
                    return Some(result)
                } else {
                    return None
                }
            }
        };
        match expression {
            Expression::Wildcard => {},
            Expression::Literal(l) if *l == next_char => {},
            Expression::Digit if next_char.is_ascii_digit() => {},
            Expression::Alphanumeric if next_char.is_ascii_alphanumeric() => {},
            Expression::Group(g) if g.chars().any(|c| c == next_char) => {},
            Expression::NegativeGroup(g) if !g.chars().any(|c| c == next_char) => {},
            Expression::End => return None,
            _ => return None
        }
    }
    Some(result)
}

fn match_pattern(input_line: &str, pattern: &Vec<Expression>) -> Option<Vec<u8>> {
    for i in 0..input_line.chars().count()  {
        let mut input_chars = input_line[i..].chars();
        match pattern[0] {
            Expression::Start => return match_pattern_from(&mut input_chars, pattern, i),
            _ => {
                match match_pattern_from(&mut input_chars, pattern, i) {
                    Some(r) => return Some(r),
                    None => {}
                }
            }
        }
    }
    None
}

fn print_result(input: String, match_idx: Vec<u8>) {
    let mut j = 0;
    let input_chars: Vec<_> = input.chars().collect();
    for i in 0..input_chars.len() {
        let next_char = String::from(input_chars[i]);
        if j < match_idx.len() && i == match_idx[j].into() {
            print!("{}", Colour::Red.bold().paint(next_char));
            j += 1;
        } else {
            print!("{}", next_char);
        }
    }
    io::stdout().flush().unwrap();
}

fn main() {
    if env::args().count() != 2 {
        println!("Wrong input.\nUsage: rust-grep pattern");
        exit(1);
    }

    let pattern = env::args().nth(1).unwrap();
    let expressions = pattern_to_expressions(&pattern);
    let mut input: String = String::new();
    while io::stdin().read_line(&mut input).expect("Error while tyring to read input") > 0 {
        match match_pattern(&input, &expressions) {
            Some(result) => print_result(input, result),
            None => {}
        }
        input = String::new();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_literals() {
        let expressions = pattern_to_expressions(&"d");
        assert_eq!(match_pattern(&"d", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"f", &expressions), None);
    }

    #[test]
    fn match_digits() {
        let expressions = pattern_to_expressions(&"\\d");
        assert_eq!(match_pattern(&"8", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"f", &expressions), None);
    }

    #[test]
    fn match_alphanumeric() {
        let expressions = pattern_to_expressions(&"\\w");
        assert_eq!(match_pattern(&"8", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"f", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"*", &expressions), None);
    }

    #[test]
    fn match_group() {
        let expressions = pattern_to_expressions(&"[abc]");
        assert_eq!(match_pattern(&"a", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"b", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"*", &expressions), None);
        assert_eq!(match_pattern(&"e", &expressions), None);
    }

    #[test]
    fn match_negative_group() {
        let expressions = pattern_to_expressions(&"[^abc]");
        assert_eq!(match_pattern(&"a", &expressions), None);
        assert_eq!(match_pattern(&"b", &expressions), None);
        assert_eq!(match_pattern(&"*", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"e", &expressions), Some(vec!(0)));
    }

    #[test]
    fn match_literal_and_group() {
        let expressions = pattern_to_expressions(&"a[abc]");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"aa", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"qa", &expressions), None);
        assert_eq!(match_pattern(&"qr", &expressions), None);
    }

    #[test]
    fn match_literal_and_digit() {
        let expressions = pattern_to_expressions(&"\\d apples");
        assert_eq!(match_pattern(&"2 apples", &expressions), Some(vec!(0, 1, 2, 3, 4, 5, 6, 7)));
        assert_eq!(match_pattern(&"2 apples ha!", &expressions), Some(vec!(0, 1, 2, 3, 4, 5, 6, 7)));
        assert_eq!(match_pattern(&"2 apple", &expressions), None);
        assert_eq!(match_pattern(&"2apples", &expressions), None);
        assert_eq!(match_pattern(&"n apples", &expressions), None);
        assert_eq!(match_pattern(&"2 organges", &expressions), None);
    }

    #[test]
    fn match_with_more_text_at_the_end() {
        let expressions = pattern_to_expressions(&"\\d apple");
        assert_eq!(match_pattern(&"2 apples", &expressions), Some(vec!(0, 1, 2, 3, 4, 5, 6)));
    }

    #[test]
    fn match_with_more_text_at_the_beginning() {
        let expressions = pattern_to_expressions(&"\\d apple");
        assert_eq!(match_pattern(&"may I have 2 apples?", &expressions), Some((11..18).collect()));
    }

    #[test]
    fn match_start_anchor() {
        let expressions = pattern_to_expressions(&"^ab");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"abc", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"aabc", &expressions), None);
        assert_eq!(match_pattern(&"rabc", &expressions), None);
    }

    #[test]
    fn match_end_anchor() {
        let expressions = pattern_to_expressions(&"ab$");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"cab", &expressions), Some(vec!(1, 2)));
        assert_eq!(match_pattern(&"aabc", &expressions), None);
        assert_eq!(match_pattern(&"abc", &expressions), None);
    }

    #[test]
    fn match_start_and_end_anchor() {
        let expressions = pattern_to_expressions(&"^ab$");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"cab", &expressions), None);
        assert_eq!(match_pattern(&"abc", &expressions), None);
    }

    #[test]
    fn match_wildcard() {
        let expressions = pattern_to_expressions(&".b");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
    }

    #[test]
    fn match_one_or_more() {
        let expressions = pattern_to_expressions(&"^a+b");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"aab", &expressions), Some(vec!(0, 1, 2)));
        assert_eq!(match_pattern(&"acb", &expressions), None);
        assert_eq!(match_pattern(&"cb", &expressions), None);
    }

    #[test]
    fn match_zero_or_more() {
        let expressions = pattern_to_expressions(&"a*b");
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"aab", &expressions), Some(vec!(0, 1, 2)));
        assert_eq!(match_pattern(&"b", &expressions), Some(vec!(0)));
    
        let other_expressions = pattern_to_expressions(&"a[bc]*d");
        assert_eq!(match_pattern(&"ad", &other_expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"cd", &other_expressions), None);
        assert_eq!(match_pattern(&"acd", &other_expressions), Some(vec!(0, 1, 2)));
    }

    #[test]
    fn match_alternations() {
        let expressions = pattern_to_expressions(&"(a|b)c");
        assert_eq!(match_pattern(&"ac", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"bc", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"ab", &expressions), None);
        assert_eq!(match_pattern(&"cc", &expressions), None);
    }

    #[test]
    fn match_alternations_with_repetition() {
        let expressions = pattern_to_expressions(&"^(a|b)*$");
        assert_eq!(match_pattern(&"a", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"b", &expressions), Some(vec!(0)));
        assert_eq!(match_pattern(&"ab", &expressions), Some(vec!(0, 1)));
        assert_eq!(match_pattern(&"ac", &expressions), None);
    }
}
