use std::env::Args;

use color_eyre::eyre::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgToken {
    // short or long flags with no value
    Flag { name: String },

    // short or long flags with an attached value
    Param { name: String, value: String },

    // value with no associated flag
    Constant(String),

    // terminator arg ('--') to indicate to stop parsing and treat the rest as all constants, used for forwarding values to command
    Terminator,
}

pub fn tokenize_args(args: Args) -> Result<Vec<ArgToken>> {
    let args = args.collect::<Vec<_>>()[1..]
        .iter()
        .map(|i| i.to_owned())
        .collect::<Vec<_>>();

    let tokens = args
        .iter()
        .map(|i| ArgToken::Constant(i.clone()))
        .collect::<Vec<_>>();

    Ok(tokens)
}
