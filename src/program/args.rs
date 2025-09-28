use std::env::Args;

use color_eyre::eyre::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgToken {
    // short or long flags with no value
    Flag(String),

    // short or long flags with an attached value
    Param(String, String),

    // value with no associated flag
    Constant(String),

    // terminator arg ('--') to indicate to stop parsing and treat the rest as all constants, used for forwarding values to command
    Terminator,
}

pub fn tokenize_args(args: Args) -> Result<Vec<ArgToken>> {
    let mut args = args.into_iter().skip(1).peekable();
    let mut tokens = Vec::new();
    let mut terminator_processed = false;

    while let Some(arg) = args.next() {
        if terminator_processed {
            tokens.push(ArgToken::Constant(arg));
        } else if arg == "--" {
            tokens.push(ArgToken::Terminator);
            terminator_processed = true;
        } else if arg.starts_with('-') {
            // TODO: it would be good to allow clustered short flags to be expanded (ie, '-abc' becomes '-a -b -c')

            let trimmed_arg = arg.trim_start_matches('-');
            let mut split_args = trimmed_arg.split('=').peekable();

            let flag = split_args.next().unwrap().to_string();
            let value: Option<String> = if split_args.peek().is_some() {
                Some(split_args.collect())
            } else if let Some(next_arg) = args.peek()
                && !next_arg.starts_with('-')
            {
                Some(args.next().unwrap())
            } else {
                None
            };

            if let Some(value) = value {
                tokens.push(ArgToken::Param(flag, value));
            } else {
                tokens.push(ArgToken::Flag(flag));
            }
        } else {
            tokens.push(ArgToken::Constant(arg));
        }
    }

    Ok(tokens)
}
