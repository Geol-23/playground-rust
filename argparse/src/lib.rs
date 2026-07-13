#![allow(warnings)]
// Standard
use std::collections::HashMap;
use std::convert::AsRef;
use std::env;
use std::fmt;
use std::iter::Iterator;
use std::marker::PhantomData;

// Internal
mod action;
mod arg_table;
mod argument;
mod error;

pub use action::{
    ArgumentAction,
};
use arg_table::ArgumentTable;
pub use argument::{
    ArgumentNumber,
    _BuildsArgument,
};
use argument::{
    ArgumentBuilder,
    Argument,
    ArgumentRef,
    PositionalArgument,
    OptionalArgument,
};
pub use error::ArgParseError;




struct ParserState<'a> {
    arg: ArgumentRef<'a>,
    arg_name: &'a str,
    argc: usize,
    pos_idx: usize,
    values: Vec<&'a str>,
}
impl<'a> ParserState<'a> {
    fn new<'b>(arg: &'b str, parser: &'a ArgumentParser) -> Self {
        match parser.optional_lookup.get(arg) {
            Some(v) => {
                let (arg_name, arg) = &parser.optional[*v];
                ParserState { arg: arg.into(), arg_name, argc: 0, pos_idx: 0, values: Vec::new() }
            },
            None => {
                let (arg_name, arg) = &parser.positional[0];
                ParserState { arg: arg.into(), arg_name, argc: 0, pos_idx: 1, values: Vec::new() }
            },
        }
    }
    fn push_value(&mut self, value: &'a str) {
        self.values.push(value);
        self.argc += 1;
    }
    fn flush(&mut self, table: &mut ArgumentTable) {
        self.arg.core().action.act(table, self.arg_name, self.arg, &self.values);
        self.values.clear();
    }
    fn switch(&mut self, table: &mut ArgumentTable, arg_name: &'a str, arg: ArgumentRef<'a>) {
        self.flush(table);
        self.arg = arg;
        self.arg_name = arg_name;
        self.argc = 0;
    }
    fn update(&mut self, table: &mut ArgumentTable, token: &'a str, parser: &'a ArgumentParser) -> Result<(), ArgParseError> {
        use ArgParseError as ArgE;
        if let Some(&i) = parser.optional_lookup.get(token) { // Is a flag
            if !self.arg.core().nargs.is_finished(self.argc) { // Is finished
                return Err(ArgE::UnexpectedOptional(token.to_string()));
            }
            let (name, arg) = &parser.optional[i];
            self.switch(table, name, arg.into());
            Ok(())
        } else if self.arg.core().nargs.is_finished(self.argc) {
            let (name, arg) = parser.positional.get(self.pos_idx)
                .ok_or_else(|| ArgE::UnexpectedPositional(token.to_string()))?;
            self.pos_idx += 1;
            self.switch(table, name, arg.into());
            self.push_value(token);
            Ok(())
        } else {
            self.push_value(token);
            Ok(())
        }
    }
}

fn is_flag<S>(s: S) -> bool
where
    S: AsRef<str>
{
    s.as_ref().starts_with("-")
}
fn is_single_flag<S>(s: S) -> bool
where
    S: AsRef<str>
{
    !s.as_ref().starts_with("--") && s.as_ref().starts_with("-")
}


pub struct ArgumentParser {
    command: HashMap<String, ArgumentParser>,
    positional: Vec<(String, PositionalArgument)>,
    optional: Vec<(String, OptionalArgument)>,
    optional_lookup: HashMap<String, usize>,
    help: String,
}
impl ArgumentParser {
    pub fn builder() -> ArgumentParserBuilder {
        ArgumentParserBuilder::new()
    }
    pub fn parse(self: &Self) -> Result<ArgumentTable, ArgParseError> { self.parse_from(&env::args().collect::<Vec<_>>()) }
    pub fn parse_from<S>(self: &Self, arguments: &[S]) -> Result<ArgumentTable, ArgParseError>
    where
        S: AsRef<str>
    {
        use ArgParseError as ArgE;
        let mut _arguments: Vec::<String> = Vec::new();
        for s in arguments {
            if is_single_flag(s) {
                for c in s.as_ref().strip_prefix('-') {
                    _arguments.push(format!("-{c}"))
                }
                continue;
            }
            _arguments.push(s.as_ref().to_string())
        }
        let mut arg_iter = _arguments.iter();

        let mut root_table = if let Some(arg) = arg_iter.next() {
            ArgumentTable::new(arg)
        } else {
            return Err(ArgE::ExpectedCommand(
                vec![env::args().next().unwrap()] // Fetches the name of the program
            ));
        };
        let mut base_parser = self;
        let mut base_table = &mut root_table;

        for (i, item) in arg_iter.by_ref().enumerate() {
            if base_parser.command.contains_key(item) {
                base_parser = base_parser.command.get(item).unwrap();
                base_table.cmd = Some(Box::new(ArgumentTable::new(&item)));
                base_table = base_table.cmd.as_mut().unwrap();
            } else {
                break
            }
        }

        let mut state = match arg_iter.next() {
            Some(argv) => {
                ParserState::new(argv.as_ref(), base_parser)
            },
            None => { // No parameters for command
                if let Some((_, arg)) = base_parser.positional.first() {
                    return Err(ArgE::ExpectedPositional(
                        base_parser.positional[0].1.name.clone()
                    ));
                }
                if !base_parser.command.is_empty() {
                    return Err(ArgE::ExpectedCommand(
                        base_parser.command
                            .keys()
                            .map(|k| k.clone())
                            .collect()
                    ))
                }
                if base_parser.optional.iter().any(|(name, arg)| {arg.required}) {
                    return Err(ArgE::RequiresFlags(base_parser.optional
                        .iter()
                        .filter_map(|(name, arg)| arg.required.then(|| arg.flags.clone()))
                        .collect()
                    ));
                }
                return Ok(root_table);
            },
        };

        for token in arg_iter {
            // Update the state
            if !token.is_ascii() { return Err(ArgE::NonASCIIInput(token.to_string())); }
            state.update(base_table, &token, base_parser)?;
        }
        state.flush(base_table);

        Ok(root_table)
    }
}



pub struct ArgumentParserBuilder {
    command_table: HashMap<String, ArgumentParser>,
    positional: Vec<(String, PositionalArgument)>,
    optional: HashMap<String, OptionalArgument>,
    help: String,
}

impl ArgumentParserBuilder {
    fn new() -> Self {
        Self {
            command_table: HashMap::new(),
            positional: Vec::new(),
            optional: HashMap::new(),
            help: String::new(),
        }
    }
    pub fn build(mut self: Self) -> ArgumentParser {
        let mut optional_lookup = HashMap::<String, usize>::new();
        let mut optional = Vec::<(String, OptionalArgument)>::new();

        for (i, e) in self.optional.drain().enumerate() {
            for argname in &e.1.flags {
                optional_lookup.insert(argname.clone(), i);
            }
            optional.push(e);
        }

        ArgumentParser {
            positional: self.positional,
            optional,
            optional_lookup,
            help: self.help,
            command: self.command_table,
        }
    }
    pub fn command<F: FnOnce(ArgumentParserBuilder) -> ArgumentParserBuilder>(mut self: Self, name: impl Into<String>, command: F) -> Self {
        let name = name.into();
        if self.command_table.contains_key(&name) { panic!("Duplicate command name: '{}'", name); }

        let mut command_builder = ArgumentParserBuilder::new();
        self.command_table.insert(name, command(command_builder).build());

        self
    }
    pub fn arg<T: _BuildsArgument, F: FnOnce(ArgumentBuilder) -> T>(mut self: Self, dest: impl Into<String>, argument: F) -> Self {
        let dest = dest.into();
        if self.positional.iter().find(|(n, _)| n == &dest).is_some() || self.optional.contains_key(&dest)  { panic!("Duplicate argument dest: '{}'", dest); }

        let mut arg_builder = ArgumentBuilder::new(&dest);
        let argument = argument(arg_builder).build();

        match argument {
            Argument::Positional(arg) => { self.positional.push((dest, arg)); },
            Argument::Optional(arg) => { self.optional.insert(dest, arg); },
        };

        self
    }
    pub fn help(mut self: Self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }
}
