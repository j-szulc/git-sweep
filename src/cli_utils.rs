use crate::Error;
use multipeek::multipeek;
use std::fmt::Display;
use std::process::{Command, Stdio};

#[allow(dead_code)] // For future use
pub(crate) fn inquire_select<'a, T>(prompt: &str, options: &'a Vec<(&str, T)>) -> &'a T {
    let options_str = options.iter().map(|x| x.0).collect::<Vec<_>>();
    let selected = inquire::Select::new(prompt, options_str)
        .prompt()
        .unwrap()
        .to_string();
    let selected = options.iter().filter(|x| x.0 == selected).next().unwrap();
    &selected.1
}

pub(crate) fn bool_to_checkmark(b: bool) -> &'static str {
    if b {
        "✅"
    } else {
        "❌"
    }
}

pub(crate) fn print_subsection<Item: Display, Container: IntoIterator<Item = Item>>(
    items: Container,
    limit: usize,
    indent: usize,
) {
    let mut items = multipeek(items.into_iter());
    let mut count = 0;
    while let Some(item) = items.next() {
        if count >= limit && items.peek_nth(2).is_some() {
            println!("{}... {} more", " ".repeat(indent), items.count());
            break;
        }
        // To remove warning from Rust analyzer
        let item: Item = item;
        println!("{}{}", " ".repeat(indent), item);
        count += 1;
    }
}

pub(crate) fn which(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .stdout(Stdio::null())
        .status()
        .unwrap()
        .success()
}

pub(crate) fn check_verbose<T: std::fmt::Display>(
    items: Vec<T>,
    checker: impl Fn(T) -> Result<bool, Error>,
) -> (bool, Vec<String>) {
    let mut all = true;
    let mut msgs = Vec::new();
    for item in items {
        let result = checker(item);
        let result_bool = result.unwrap_or(false);
        all = all && result_bool;
        let error_msg = result
            .err()
            .map(|x| format!(" - {x}"))
            .unwrap_or("".to_string());
        msgs.push(format!(
            "{}: {}{}",
            bool_to_checkmark(result_bool),
            item,
            error_msg
        ));
    }
    (all, msgs)
}
