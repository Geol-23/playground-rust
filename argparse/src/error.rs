// Standard imports
use std::fmt;

// Internal imports
use super::argument::{
    PositionalArgument,
    OptionalArgument,
    ArgumentNumber
};

type Flags = Vec<String>;

#[derive(Debug)]
pub enum ArgParseError {
    UnexpectedPositional(String),
    UnexpectedOptional(String),
    ExpectedPositional(String),
    ExpectedArguments(Flags, ArgumentNumber),
    ExpectedCommand(Vec<String>),
    RequiresFlags(Vec<Flags>),
    NonASCIIInput(String),
    InvalidChoice(String, Vec<String>),
    UndefinedConstant(String),

    URStateArgNotSet, // unreachable
}

impl fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgParseError::UnexpectedPositional(value) => write!(f, "unexpected positional argument '{}'", value),
            ArgParseError::UnexpectedOptional(flag) => write!(f, "unexpected optional argument '{}'", flag),
            ArgParseError::ExpectedPositional(arg_name) => write!(f, "missing positional argument {}", arg_name),
            ArgParseError::ExpectedArguments(flags, nargs) => write!(f, "argument(s) {}: expected {}", format!("[{}]", flags.join(", ")), nargs),
            ArgParseError::ExpectedCommand(cmds) => write!(f, "command name is required: {{{}}}", cmds.join(", ")),
            ArgParseError::RequiresFlags(flags) => write!(f, "the following flags are required: {{{}}}",
                flags
                    .iter()
                    .map(|flag| format!("[{}]", flags.join(", ")))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ArgParseError::NonASCIIInput(value) => write!(f, "non-ascii characters encountered: {}", value),
            ArgParseError::InvalidChoice(choice, choices) => write!(f, "invalid choice: {}, choose from: [{}]", choice, choices.join(", ")),
            ArgParseError::UndefinedConstant(name_or_flags) => write!(f, "no constant defined for {}", name_or_flags),

            ArgParseError::URStateArgNotSet => write!(f, "parameter wasn't a command but argument wasn't set")
        }
    }
}
impl std::error::Error for ArgParseError {}
