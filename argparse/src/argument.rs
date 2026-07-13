// Standard imports
use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;

// Internal imports
use super::action::{
    ArgumentAction,
};

pub enum Argument {
    Positional(PositionalArgument),
    Optional(OptionalArgument),
}
impl Argument {
    pub fn core(self: &Self) -> &ArgumentCore {
        match self {
            Argument::Positional(pos) => &pos.core,
            Argument::Optional(opt) => &opt.core,
        }
    }
    pub fn label(self: &Self) -> String {
        match self {
            Argument::Positional(pos) => pos.name.clone(),
            Argument::Optional(opt) => opt.flags.join(", "),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArgumentRef<'a> {
    Positional(&'a PositionalArgument),
    Optional(&'a OptionalArgument),
}
impl<'a> ArgumentRef<'a> {
    pub fn core(self: &Self) -> &'a ArgumentCore {
        match self {
            ArgumentRef::Positional(pos) => &pos.core,
            ArgumentRef::Optional(opt) => &opt.core,
        }
    }
    pub fn label(self: &Self) -> String {
        match self {
            ArgumentRef::Positional(pos) => pos.name.clone(),
            ArgumentRef::Optional(opt) => opt.flags.join(", "),
        }
    }
}

impl<'a> From<&'a PositionalArgument> for ArgumentRef<'a> {
    fn from(p: &'a PositionalArgument) -> Self { ArgumentRef::Positional(p) }
}
impl<'a> From<&'a OptionalArgument> for ArgumentRef<'a> {
    fn from(o: &'a OptionalArgument) -> Self { ArgumentRef::Optional(o) }
}
impl<'a> From<&'a Argument> for ArgumentRef<'a> {
    fn from(arg: &'a Argument) -> Self {
        match arg {
            Argument::Positional(a) => ArgumentRef::Positional(a),
            Argument::Optional(a) => ArgumentRef::Optional(a),
        }
    }
}

trait IntoVecString {
    fn into(self) -> Vec<String>;
}

impl IntoVecString for String {
    fn into(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoVecString for &str {
    fn into(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl<T: Into<String>> IntoVecString for Vec<T> {
    fn into(self) -> Vec<String> {
        self.into_iter().map(|v| v.into()).collect()
    }
}

pub trait _BuildsArgument {
    fn build(self: Self) -> Argument;
}

pub struct StatePositional;
pub struct StateOptional;
#[derive(Debug)]
pub struct ArgumentBuilder<S=StatePositional> {
    argument: AbstractArgument,
    _marker: PhantomData<S>,
}
impl ArgumentBuilder {
    pub fn new(dest: &str) -> ArgumentBuilder<StatePositional> {
        ArgumentBuilder::<_> {
            argument: AbstractArgument::new(dest),
            _marker: PhantomData::<_>,
        }
    }
}
impl<S> ArgumentBuilder<S> {
    fn transition<NewState>(self) -> ArgumentBuilder<NewState> {
        ArgumentBuilder::<_> {
            argument: self.argument,
            _marker: PhantomData,
        }
    }
    pub fn name(mut self: Self, name: impl Into<String>) -> ArgumentBuilder<StatePositional> {
        self.argument.argname = ArgumentIdentifier::Name(name.into());
        self.transition::<StatePositional>()
    }
    pub fn flags(mut self: Self, flags: impl IntoVecString) -> ArgumentBuilder<StateOptional> {
        let mut flags = flags.into();
        for flag in &flags {
            if !flag.starts_with("-") { panic!("Flag must start with '-'"); }
            if !flag.starts_with("--") && flag.len() != 2 { panic!("Flag with single '-' can only contain 1 character"); }
            if !flag.is_ascii() { panic!("Flag must only contain ascii characters"); }
        }

        match &mut self.argument.argname {
            ArgumentIdentifier::Flag(vec) => vec.append(&mut flags),
            ArgumentIdentifier::Name(_) => { self.argument.argname = ArgumentIdentifier::Flag(flags); },
        }

        self.argument.required = false;
        self.transition::<StateOptional>()
    }
    pub fn action(mut self: Self, action: ArgumentAction) -> Self {
        self.argument.core.action = action;
        self
    }
    pub fn nargs(mut self: Self, nargs: ArgumentNumber) -> Self {
        self.argument.core.nargs = nargs;
        self
    }
    pub fn constant(mut self: Self, constant: impl IntoVecString) -> Self {
        self.argument.core.constant = Some(constant.into());
        self
    }
    pub fn choices(mut self: Self, choices: impl IntoVecString) -> Self {
        let mut choices: Vec<String> = choices.into();
        for choice in &choices {
            if choice.starts_with("-") { panic!("Choice cannot start with '-'"); }
            if !choice.is_ascii() { panic!("Choice must only contain ascii characters"); }
        }
        self.argument.core.choices = Some(choices.into_iter().collect());
        self
    }
    pub fn help(mut self: Self, help: impl Into<String>) -> Self {
        self.argument.core.help = help.into();
        self
    }
}
impl _BuildsArgument for ArgumentBuilder<StatePositional> {
    fn build(self: Self) -> Argument {
        Argument::Positional(
            self.argument.into_positional()
        )
    }
}
impl _BuildsArgument for ArgumentBuilder<StateOptional> {
    fn build(self: Self) -> Argument {
        Argument::Optional(
            self.argument.into_optional()
        )
    }
}
impl ArgumentBuilder<StateOptional> {
    pub fn build(self: Self) -> Argument {
        Argument::Optional(
            self.argument.into_optional()
        )
    }
    pub fn default(mut self: Self, default: impl IntoVecString) -> Self {
        self.argument.default = Some(default.into());
        self
    }
    pub fn required(mut self: Self) -> Self {
        self.argument.required = true;
        self
    }
    pub fn not_required(mut self: Self) -> Self {
        self.argument.required = false;
        self
    }
}

#[derive(Debug, Clone)]
enum ArgumentIdentifier {
    Name(String),
    Flag(Vec<String>)
}

#[derive(Debug, Clone, Copy)]
pub enum ArgumentNumber {
    N(usize),
    NOrLess(usize),
    NOrMore(usize),
}
impl ArgumentNumber {
    pub const OneOrMore: Self = Self::NOrMore(1);
    pub const OneOrNone: Self = Self::NOrLess(1);
    pub const ZeroOrMore: Self = Self::NOrMore(0);
}
impl fmt::Display for ArgumentNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgumentNumber::N(i) => write!(f, "={}", i),
            ArgumentNumber::NOrLess(i) => write!(f, "<={}", i),
            ArgumentNumber::NOrMore(i) => write!(f, ">={}", i),
        }
    }
}
impl ArgumentNumber {
    pub fn is_valid(self: &Self, count: usize) -> bool {
        match self {
            ArgumentNumber::N(i) => count == *i,
            ArgumentNumber::NOrLess(i) => count <= *i,
            ArgumentNumber::NOrMore(i) => count >= *i,
        }
    }
    pub fn is_finished(self: &Self, count: usize) -> bool {
        match self {
            ArgumentNumber::N(i) => count == *i,
            ArgumentNumber::NOrLess(i) => count <= *i,
            ArgumentNumber::NOrMore(i) => count >= *i,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentCore {
    pub action: ArgumentAction,
    pub nargs: ArgumentNumber,
    pub constant: Option<Vec<String>>,
    pub choices: Option<HashSet<String>>,
    pub help: String,
}

#[derive(Debug, Clone)]
pub struct PositionalArgument {
    pub name: String,
    pub core: ArgumentCore,
}

#[derive(Debug, Clone)]
pub struct OptionalArgument {
    pub flags: Vec<String>,
    pub default: Option<Vec<String>>,
    pub required: bool,
    pub core: ArgumentCore,
}

#[derive(Debug)]
struct AbstractArgument {
    argname: ArgumentIdentifier,
    required: bool,
    default: Option<Vec<String>>,
    core: ArgumentCore,
}
impl AbstractArgument {
    fn new(name: &str) -> AbstractArgument {
        AbstractArgument {
            argname: ArgumentIdentifier::Name(name.to_string()),
            default: None,
            required: true,
            core: ArgumentCore {
                action: ArgumentAction::Store,
                nargs: ArgumentNumber::N(1),
                constant: None,
                choices: None,
                help: String::new(),
            },
        }
    }
    fn into_positional(self: Self) -> PositionalArgument {
        let name = match self.argname {
            ArgumentIdentifier::Name(n) => n,
            ArgumentIdentifier::Flag(_) => { panic!("argument identifier not a name"); }
        };
        let core = self.core;
        PositionalArgument {
            name,
            core,
        }
    }

    fn into_optional(self: Self) -> OptionalArgument {
        let flags = match self.argname {
            ArgumentIdentifier::Name(_) => { panic!("argument identifier not a flag(s)"); },
            ArgumentIdentifier::Flag(f) => f,
        };
        OptionalArgument {
            flags,
            default: self.default,
            required: self.required,
            core: self.core,
        }
    }
}
