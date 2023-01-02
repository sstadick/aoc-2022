use std::{
    collections::VecDeque,
    fmt,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    process::exit,
};

use clap::Parser;

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day11a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day11a {
    fn main(&self) -> Result<(), DynError> {
        let mut reader = BufReader::new(File::open(&self.input).unwrap());

        let mut input = String::new();
        let bytes_read = reader.read_to_string(&mut input).unwrap();
        let (remainder, mut monkeys) = monkey_parser::parse_monkeys(&input).unwrap();

        for i in 0..20 {
            let monkey_len = monkeys.len();
            let slice = monkeys.as_mut_slice();
            for monkey_index in 0..monkey_len {
                let (left, monkey, right) = split_on(slice, monkey_index);

                while let Some(item) = monkey.items.pop_front() {
                    let new = (*monkey.operation)(item); // Increase worry as item is inspected
                    let new = Item(new.0 / 3); // Decrease worry as item is moved on from
                    let index = if new.0 % monkey.test_number == 0 {
                        monkey.test_true_branch
                    } else {
                        monkey.test_false_branch
                    };
                    if index < monkey_index {
                        left[index].items.push_back(new);
                    } else {
                        right[index - (left.len() + 1)].items.push_back(new);
                    }

                    monkey.total_inspections += 1;
                }
            }
        }

        monkeys.sort_by(|a, b| b.total_inspections.cmp(&a.total_inspections));
        eprintln!("{}", monkeys[0].total_inspections * monkeys[1].total_inspections);
        for monkey in monkeys {
            eprintln!("{}: {}", monkey.id, monkey.total_inspections);
        }
        Ok(())
    }
}

/// Split a slice on an index, return the slice before that index, the value at the index,
/// and the slice after the index.
pub fn split_on<T>(slice: &mut [T], index: usize) -> (&mut [T], &mut T, &mut [T]) {
    // Here we have to do some work to convince the borrowchecker that we will
    // not be borrowing the same itemmutably twice
    let (left, right) = slice.split_at_mut(index);
    let (element, right) = right.split_at_mut(1);
    let element = &mut element[0];
    (left, element, right)
}

pub struct Monkey {
    /// The id of the monkey.
    pub id: u8,
    /// Total number of inspections this monkey has done so far.
    pub total_inspections: usize,
    /// The items currently held by the monkey.
    pub items: VecDeque<Item>,
    /// The function to apply to each item when it's picked up for inspection
    pub operation: Box<dyn Fn(Item) -> Item>,
    /// The number to divide the inner worry level by
    pub test_number: u64,
    /// The monkey to throw to if the test was true
    pub test_true_branch: usize,
    /// The monkey to throw to if the test was false
    pub test_false_branch: usize,
}

impl fmt::Debug for Monkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Monkey")
            .field("id", &self.id)
            .field("total_inspections", &self.total_inspections)
            .field("items", &self.items)
            .field("test_number", &self.test_number)
            .field("test_true_branch", &self.test_true_branch)
            .field("test_false_branch", &self.test_false_branch)
            .finish()
    }
}

pub mod monkey_parser {
    use std::collections::VecDeque;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{self, line_ending, multispace0},
        combinator::{self, complete, iterator, map},
        error::ParseError,
        multi::{many0, separated_list1},
        sequence::{delimited, terminated},
        IResult,
    };

    use super::{Item, Monkey};

    enum WorryValue {
        Old,
        New,
        Value(u64),
    }

    enum WorryOp {
        Add,
        Mul,
        Div,
        Sub,
    }

    pub(crate) fn parse_monkeys(input: &str) -> IResult<&str, Vec<Monkey>> {
        many0(parse_monkey)(input)
    }

    /// Parsed Monkey
    /// That funky Monkey
    /// Parsed Monkey junkie
    /// That funky Monkey
    /// Parsed
    fn parse_monkey(input: &str) -> IResult<&str, Monkey> {
        // Note: throught this newlines are consumed by the `ws` chomp functions that eats newlines
        let (input, id) = parse_monkey_id(input)?;
        let (input, starting_items) = parse_starting_items(input)?;
        let (input, op) = parse_operation(input)?;
        let (input, (test_number, test_true_branch, test_false_branch)) = parse_test(input)?;
        Ok((
            input,
            Monkey {
                id,
                total_inspections: 0,
                items: starting_items,
                operation: Box::new(op),
                test_number,
                test_true_branch: test_true_branch as usize,
                test_false_branch: test_false_branch as usize,
            },
        ))
    }

    /// Parse the "Monkey 5:\n" line
    fn parse_monkey_id(input: &str) -> IResult<&str, u8> {
        let (input, _) = tag("Monkey ")(input)?;
        let (input, id) = complete::u8(input)?;
        let (input, _) = tag(":")(input)?;
        let (input, _) = complete::line_ending(input)?;
        Ok((input, id))
    }

    /// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
    /// trailing whitespace, returning the output of `inner`.
    fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
        inner: F,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, E>,
    {
        delimited(multispace0, inner, multispace0)
    }

    fn chomp_and_parse_u64(input: &str) -> IResult<&str, u64> {
        ws(complete::u64)(input)
    }

    /// Parse the starting items the monkey has
    fn parse_starting_items(input: &str) -> IResult<&str, VecDeque<Item>> {
        let (input, _) = complete::multispace0(input)?;
        let (input, _) = tag("Starting items:")(input)?;
        let (input, items) = separated_list1(tag(","), chomp_and_parse_u64)(input)?;
        let queue = items.into_iter().map(Item).collect();
        Ok((input, queue))
    }

    fn parse_worry_value(input: &str) -> IResult<&str, WorryValue> {
        let old = map(ws(tag("old")), |_| WorryValue::Old);
        let new = map(ws(tag("new")), |_| WorryValue::New);
        let value = map(ws(complete::u64), WorryValue::Value);
        alt((old, new, value))(input)
    }

    fn parse_worry_op(input: &str) -> IResult<&str, WorryOp> {
        let add = map(ws(tag("+")), |_| WorryOp::Add);
        let sub = map(ws(tag("-")), |_| WorryOp::Sub);
        let mul = map(ws(tag("*")), |_| WorryOp::Mul);
        let div = map(ws(tag("/")), |_| WorryOp::Div);
        alt((add, sub, mul, div))(input)
    }

    fn parse_operation(input: &str) -> IResult<&str, impl Fn(Item) -> Item> {
        let (input, _) = ws(tag("Operation: new ="))(input)?;
        let (input, val1) = parse_worry_value(input)?;
        let (input, op) = parse_worry_op(input)?;
        let (input, val2) = parse_worry_value(input)?;

        let func = move |item: Item| {
            let num1 = match val1 {
                WorryValue::Old => item.0,
                WorryValue::New => panic!("New is not accessible in expression"),
                WorryValue::Value(value) => value,
            };
            let num2 = match val2 {
                WorryValue::Old => item.0,
                WorryValue::New => panic!("New is not accessible in expression"),
                WorryValue::Value(value) => value,
            };

            let value = match op {
                WorryOp::Add => num1 + num2,
                WorryOp::Mul => num1 * num2,
                WorryOp::Div => num1 / num2,
                WorryOp::Sub => num1 - num2,
            };
            Item(value)
        };
        Ok((input, func))
    }

    fn parse_test(input: &str) -> IResult<&str, (u64, u64, u64)> {
        let (input, _) = ws(tag("Test: divisible by"))(input)?;
        let (input, test_number) = ws(complete::u64)(input)?;
        let (input, _) = ws(tag("If true: throw to monkey"))(input)?;
        let (input, true_branch) = ws(complete::u64)(input)?;
        let (input, _) = ws(tag("If false: throw to monkey"))(input)?;
        let (input, false_branch) = ws(complete::u64)(input)?;
        Ok((input, (test_number, true_branch, false_branch)))
    }
}

/// An item held by a monkey.
///
/// The inner number represents the current worry level for that item.
#[derive(Debug, Copy, Clone)]
pub struct Item(pub u64);
