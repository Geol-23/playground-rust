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
    StoreAction,
    StoreConstAction,
    StoreTrueAction,
    StoreFalseAction,
    AppendAction,
    AppendConstAction,
    ExtendAction,
    CountAction,
};
use arg_table::ArgumentTable;
pub use argument::{
    ArgumentNumber,
};
use argument::{
    ArgumentBuilder,
    PositionalOrOptional,
    PositionalArgument,
    OptionalArgument,
};
pub use error::ArgParseError;



pub struct ArgumentParser {
    command: HashMap<String, ArgumentParser>,
    positional: Vec<(String, PositionalOrOptional)>,
    optional: Vec<(String, PositionalOrOptional)>,
    optional_lookup: HashMap<String, usize>,
    help: String,
}

struct Uninitialised;
struct Initialised;
struct ParserState<'a, S=Uninitialised> {
    arg: &'a PositionalOrOptional,
    arg_name: &'a String,
    argc: usize,
    pos_idx: usize,
    _marker: PhantomData<S>,
}

impl<'a> ParserState<'a> {
    fn initialise<'b>(arg: &'b str, parser: &'a ArgumentParser) -> ParserState<'a, Initialised> {
        match parser.optional_lookup.get(arg) {
            Some(v) => {
                let (arg_name, arg) = &parser.optional[*v];
                ParserState::<Initialised> {
                   arg,
                   arg_name,
                   argc: 0,
                   pos_idx: 0,
                   _marker: PhantomData::<_>,
               }
            },
            None => {
                let (arg_name, arg) = &parser.positional[0];
                ParserState::<Initialised> {
                   arg,
                   arg_name,
                   argc: 0,
                   pos_idx: 1,
                   _marker: PhantomData::<_>,
               }
            },
        }
    }
}

impl <'a> ParserState<'a, Initialised> {
    fn get_nargs(self: &Self) -> ArgumentNumber {
        match self.arg {
            PositionalOrOptional::Positional(arg) => { arg.nargs },
            PositionalOrOptional::Optional(arg) => { arg.nargs },
        }
    }
    fn set_optional(self: &mut Self, parser: &'a ArgumentParser, i: usize) -> Result<(), ()> {
        // check if the current arg is finished
        match self.get_nargs() {
            ArgumentNumber::N(i) => { if self.argc != i { return Err(()); } },
            ArgumentNumber::NOrLess(i) => { if self.argc > i { return Err(()); } },
            ArgumentNumber::NOrMore(i) => { if self.argc < i { return Err(()); } },
        };

        // set the optional arg
        let (arg_name, op_arg) = &parser.optional[i];
        self.arg = op_arg;
        self.arg_name = arg_name;
        self.argc = 0;
        Ok(())
    }
    fn inc_or_positional<F: Fn(usize) -> bool>(self: &mut Self, parser: &'a ArgumentParser, predicate: F) -> Result<(), ()> {
        if predicate(self.argc) {
            match parser.positional.get(self.pos_idx) {
                Some((arg_name, new_arg)) => {
                    self.pos_idx += 1;
                    self.arg = new_arg;
                    self.argc = 0;
                    Ok(())
                },
                None => { Err(()) },
            }
        } else {
            self.argc += 1;
            Ok(())
        }
    }
    fn update<'b>(self: &mut Self, inp_arg: &'b str, parser: &'a ArgumentParser) -> Result<(), ArgParseError> {
        use ArgParseError as ArgE;

        match parser.optional_lookup.get(inp_arg) {
            Some(i) => { self.set_optional(parser, *i).map_err(|_| ArgE::UnexpectedOptional(inp_arg.to_string())) },
            None => {
                match self.get_nargs() {
                    ArgumentNumber::N(i) => {
                        self.inc_or_positional(parser, |count| count == i)
                            .map_err(|_| ArgE::UnexpectedPositional(inp_arg.to_string()))
                    },
                    ArgumentNumber::NOrLess(i) => {
                        self.inc_or_positional(parser, |count| count >= 1)
                            .map_err(|_| ArgE::UnexpectedPositional(inp_arg.to_string()))
                    },
                    ArgumentNumber::NOrMore(i) => {
                        self.inc_or_positional(parser, |_| false)
                            .map_err(|_| ArgE::UnexpectedPositional(inp_arg.to_string()))
                    },
                }
            },
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

        let mut arg_iter = arguments.iter()
            .flat_map(|s| -> Box<dyn Iterator<Item = String>> {
                    if let Some(rest) = s.as_ref().strip_prefix('-') {
                        Box::new(rest.chars().map(|c| format!("-{c}")))
                    } else {
                        Box::new(std::iter::once(s.as_ref().to_string()))
                    }
                });


        let mut root_table = ArgumentTable::new(
            &arg_iter
                .next()
                .ok_or(ArgE::ExpectedCommand(
                    vec![env::args().next().unwrap()]
                ))?
        );
        let mut base_parser = self;
        let mut base_table = &mut root_table;

        for (i, item) in arg_iter.by_ref().enumerate() {
            if base_parser.command.contains_key(&item) {
                base_parser = base_parser.command.get(&item).unwrap();
                base_table.cmd = Some(Box::new(ArgumentTable::new(&item)));
                base_table = base_table.cmd.as_mut().unwrap();
            } else {
                break
            }
        }

        let mut state = match arg_iter.next() {
            Some(argv) => {
                ParserState::initialise(argv.as_ref(), base_parser)
            },
            None => { // No parameters for command
                if let Some((_, arg)) = base_parser.positional.first() {
                    return Err(ArgE::ExpectedPositional(base_parser.positional[0].1.as_pos().clone()));
                }
                if !base_parser.command.is_empty() {
                    return Err(ArgE::ExpectedCommand(
                        base_parser.command
                            .keys()
                            .map(|k| k.clone())
                            .collect()
                    ))
                }
                if base_parser.optional.iter().any(|(name, arg)| {arg.as_op().required}) {
                    return Err(ArgE::RequiresFlags(base_parser.optional
                        .iter()
                        .filter_map(|(name, arg)| arg.as_op().required.then(|| arg.as_op().clone()))
                        .collect()
                    ));
                }
                return Ok(root_table);
            },
        };


        let mut group: Vec<&str> = Vec::new();
        for item in arg_iter {
            // Update the state
            if !item.is_ascii() { return Err(ArgE::NonASCIIInput(item.to_string())); }
            state.update(&item, base_parser)?;
            if is_flag(item) { continue; }

            // If arg has choices, check against and append


        }

        Ok(root_table)
    }
}



pub struct ArgumentParserBuilder {
    command_table: HashMap<String, ArgumentParser>,
    positional: Vec<(String, PositionalOrOptional)>,
    optional: HashMap<String, PositionalOrOptional>,
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
        let mut optional = Vec::<(String, PositionalOrOptional)>::new();

        for (i, e) in self.optional.drain().enumerate() {
            for argname in &e.1.as_op().flags {
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
    pub fn arg<F: FnOnce(ArgumentBuilder) -> ArgumentBuilder>(mut self: Self, dest: impl Into<String>, argument: F) -> Self {
        let dest = dest.into();
        if self.positional.iter().find(|(n, _)| n == &dest).is_some() || self.optional.contains_key(&dest)  { panic!("Duplicate argument dest: '{}'", dest); }

        let mut arg_builder = ArgumentBuilder::new().name(dest.clone());
        arg_builder = argument(arg_builder);
        let argument = arg_builder.build();

        match argument {
            PositionalOrOptional::Positional(_) => { self.positional.push((dest, argument)); },
            PositionalOrOptional::Optional(_) => { self.optional.insert(dest, argument); },
        };

        self
    }
    pub fn help(mut self: Self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }
}
