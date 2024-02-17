use std::env;
use std::io;
use std::process::exit;

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


fn match_pattern_from(input_chars: &mut std::str::Chars<'_>, pattern: &Vec<Expression>) -> bool {
    for expression_idx in 0..pattern.len() {
        let expression = &pattern[expression_idx];
        match expression {
            Expression::Start => continue,
            Expression::OneOrMore => {
                let next_expression = &pattern[expression_idx + 1];
                let mut rest = Vec::from(&pattern[(expression_idx + 1)..]);
                rest.insert(0, Expression::ZeroOrMore);
                rest.insert(0, next_expression.clone());
                return match_pattern_from(&mut input_chars.clone(), &mut rest);
            },
            Expression::ZeroOrMore => {
                let next_expression = &pattern[expression_idx + 1];
                let mut rest_with = Vec::from(&pattern[(expression_idx + 1)..]);
                rest_with.insert(0, Expression::ZeroOrMore);
                rest_with.insert(0, next_expression.clone());
                let res_with = match_pattern_from(&mut input_chars.clone(), &mut rest_with);

                let mut rest_wo = Vec::from(&pattern[(expression_idx + 2)..]);
                let res_wo = match_pattern_from(&mut input_chars.clone(), &mut rest_wo);

                return res_with || res_wo
            },
            Expression::Alternations(alternations) => {
                for alternation in alternations.into_iter() {
                    let mut rest = Vec::from(&pattern[(expression_idx + 1)..]);
                    let mut alternation_clone = alternation.clone();
                    alternation_clone.append(&mut rest);
                    if match_pattern_from(&mut input_chars.clone(), &mut alternation_clone) {
                        return true
                    }
                }
                return false
            }
            _ => {}
        }
        let next_char = match input_chars.next() {
            Some(c) => c,
            None => return *expression == Expression::End 
        };
        match expression {
            Expression::Wildcard => {},
            Expression::Literal(l) if *l == next_char => {},
            Expression::Digit if next_char.is_ascii_digit() => {},
            Expression::Alphanumeric if next_char.is_ascii_alphanumeric() => {},
            Expression::Group(g) if g.chars().any(|c| c == next_char) => {},
            Expression::NegativeGroup(g) if !g.chars().any(|c| c == next_char) => {},
            Expression::End => return false,
            _ => return false
        }
    }
    return true
}

fn match_pattern(input_line: &str, pattern: &Vec<Expression>) -> bool {
    for i in 0..input_line.chars().count()  {
        let mut input_chars = input_line[i..].chars();
        match pattern[0] {
            Expression::Start => {
                return match_pattern_from(&mut input_chars, pattern)
            },
            _ => {
                if match_pattern_from(&mut input_chars, pattern) {
                    return true
                }
            }
        }
    }
    false
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
        if match_pattern(&input, &expressions) {
            println!("match");
        } else {
            println!("no match");
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
        assert_eq!(match_pattern(&"d", &expressions), true);
        assert_eq!(match_pattern(&"f", &expressions), false);
    }

    #[test]
    fn match_digits() {
        let expressions = pattern_to_expressions(&"\\d");
        assert_eq!(match_pattern(&"8", &expressions), true);
        assert_eq!(match_pattern(&"f", &expressions), false);
    }

    #[test]
    fn match_alphanumeric() {
        let expressions = pattern_to_expressions(&"\\w");
        assert_eq!(match_pattern(&"8", &expressions), true);
        assert_eq!(match_pattern(&"f", &expressions), true);
        assert_eq!(match_pattern(&"*", &expressions), false);
    }

    #[test]
    fn match_group() {
        let expressions = pattern_to_expressions(&"[abc]");
        assert_eq!(match_pattern(&"a", &expressions), true);
        assert_eq!(match_pattern(&"b", &expressions), true);
        assert_eq!(match_pattern(&"*", &expressions), false);
        assert_eq!(match_pattern(&"e", &expressions), false);
    }

    #[test]
    fn match_negative_group() {
        let expressions = pattern_to_expressions(&"[^abc]");
        assert_eq!(match_pattern(&"a", &expressions), false);
        assert_eq!(match_pattern(&"b", &expressions), false);
        assert_eq!(match_pattern(&"*", &expressions), true);
        assert_eq!(match_pattern(&"e", &expressions), true);
    }

    #[test]
    fn match_literal_and_group() {
        let expressions = pattern_to_expressions(&"a[abc]");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"aa", &expressions), true);
        assert_eq!(match_pattern(&"qa", &expressions), false);
        assert_eq!(match_pattern(&"qr", &expressions), false);
    }

    #[test]
    fn match_literal_and_digit() {
        let expressions = pattern_to_expressions(&"\\d apples");
        assert_eq!(match_pattern(&"2 apples", &expressions), true);
        assert_eq!(match_pattern(&"2 apple", &expressions), false);
        assert_eq!(match_pattern(&"2apples", &expressions), false);
        assert_eq!(match_pattern(&"n apples", &expressions), false);
        assert_eq!(match_pattern(&"2 organges", &expressions), false);
    }

    #[test]
    fn match_with_more_text_at_the_end() {
        let expressions = pattern_to_expressions(&"\\d apple");
        assert_eq!(match_pattern(&"2 apples", &expressions), true);
    }

    #[test]
    fn match_with_more_text_at_the_beginning() {
        let expressions = pattern_to_expressions(&"\\d apple");
        assert_eq!(match_pattern(&"may I have 2 apples?", &expressions), true);
    }

    #[test]
    fn match_start_anchor() {
        let expressions = pattern_to_expressions(&"^ab");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"abc", &expressions), true);
        assert_eq!(match_pattern(&"aabc", &expressions), false);
        assert_eq!(match_pattern(&"rabc", &expressions), false);
    }

    #[test]
    fn match_end_anchor() {
        let expressions = pattern_to_expressions(&"ab$");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"cab", &expressions), true);
        assert_eq!(match_pattern(&"aabc", &expressions), false);
        assert_eq!(match_pattern(&"abc", &expressions), false);
    }

    #[test]
    fn match_start_and_end_anchor() {
        let expressions = pattern_to_expressions(&"^ab$");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"cab", &expressions), false);
        assert_eq!(match_pattern(&"abc", &expressions), false);
    }

    #[test]
    fn match_wildcard() {
        let expressions = pattern_to_expressions(&".b");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"1b", &expressions), true);
        assert_eq!(match_pattern(&"(b", &expressions), true);
    }

    #[test]
    fn match_one_or_more() {
        let expressions = pattern_to_expressions(&"a+b");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"aab", &expressions), true);
        assert_eq!(match_pattern(&"acb", &expressions), false);
        assert_eq!(match_pattern(&"cb", &expressions), false);
    }

    #[test]
    fn match_zero_or_more() {
        let expressions = pattern_to_expressions(&"a*b");
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"aab", &expressions), true);
        assert_eq!(match_pattern(&"acb", &expressions), true);
        assert_eq!(match_pattern(&"cb", &expressions), true);
        assert_eq!(match_pattern(&"b", &expressions), true);
    
        let other_expressions = pattern_to_expressions(&"a[bc]*d");
        assert_eq!(match_pattern(&"ad", &other_expressions), true);
        assert_eq!(match_pattern(&"cd", &other_expressions), false);
        assert_eq!(match_pattern(&"acd", &other_expressions), true);
    }

    #[test]
    fn match_alternations() {
        let expressions = pattern_to_expressions(&"(a|b)c");
        assert_eq!(match_pattern(&"ac", &expressions), true);
        assert_eq!(match_pattern(&"bc", &expressions), true);
        assert_eq!(match_pattern(&"ab", &expressions), false);
        assert_eq!(match_pattern(&"cc", &expressions), false);
    }

    #[test]
    fn match_alternations_with_repetition() {
        let expressions = pattern_to_expressions(&"^(a|b)*$");
        assert_eq!(match_pattern(&"a", &expressions), true);
        assert_eq!(match_pattern(&"b", &expressions), true);
        assert_eq!(match_pattern(&"ab", &expressions), true);
        assert_eq!(match_pattern(&"ac", &expressions), false);
    }
}
