// Standard imports
use std::collections::HashSet;
use std::fmt;

// Internal imports
use super::action::{
    ArgumentAction,
    StoreAction,
};

pub enum PositionalOrOptional {
    Positional(PositionalArgument),
    Optional(OptionalArgument),
}
impl PositionalOrOptional {
    pub fn core(self: &Self) -> &ArgumentCore {
        match self {
            PositionalOrOptional::Positional(pos) => pos.core,
            PositionalOrOptional::Optional(opt) => opt.core,
        }
    }
    pub fn label(self: &Self) -> String {
        match self {
            PositionalOrOptional::Positional(pos) => pos.name.clone(),
            PositionalOrOptional::Optional(opt) => opt.flags.join(", "),
        }
    }
    pub fn as_pos(self: &Self) -> &PositionalArgument {
        match self {
            PositionalOrOptional::Positional(a) => a,
            PositionalOrOptional::Optional(_) => panic!("attempted to create positional from optional")
        }
    }
    fn to_pos(self: Self) -> PositionalArgument {
        match self {
            PositionalOrOptional::Positional(a) => a,
            PositionalOrOptional::Optional(_) => panic!("attempted to create positional from optional")
        }
    }
    pub fn as_op(self: &Self) -> &OptionalArgument {
        match self {
            PositionalOrOptional::Optional(a) => a,
            PositionalOrOptional::Positional(_) => panic!("attempted to create optional from positional")
        }
    }
    fn to_op(self: Self) -> OptionalArgument{
        match self {
            PositionalOrOptional::Optional(a) => a,
            PositionalOrOptional::Positional(_) => panic!("attempted to create optional from positional")
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


struct StatePositional;
struct StateOptional;
#[derive(Debug)]
pub struct ArgumentBuilder<S=StatePositional> {
    argument: AbstractArgument,
    _marker: PhantomData<S>,
}
impl ArgumentBuilder {
    fn new(dest: &str) -> Self<StatePositional> {
        Self::<_> {
            argument: AbstractArgument::new_positional(dest),
            _marker: PhantomData::<_>,
        }
    }
}
impl<S> ArgumentBuilder<S> {
    fn transition<NewState>(self) -> Self<NewState> {
        Self {
            argument: self.argument,
            _marker: PhantomData,
        }
    }
    pub fn name(mut self: Self, name: impl Into<String>) -> Self<StatePositional> {
        self.argument.argname = ArgumentIdentifier::Name(name.into());
        self.transition::<StatePositional>()
    }
    pub fn flags(mut self: Self, flags: impl IntoVecString) -> Self<StateOptional> {
        let mut flags = flags.into();
        for flag in &flags {
            if !flag.starts_with("-") { panic!("Flag must start with '-'"); }
            if !flag.starts_with("--") && flag.len() != 2 { panic!("Flag with single '-' can only contain 1 character"); }
            if !flag.is_ascii() { panic!("Flag must only contain ascii characters"); }
        }

        match &mut self.argument.argname {
            Some(ArgumentIdentifier::Flag(vec)) => vec.append(&mut flags),
            Some(ArgumentIdentifier::Name(_)) | None => self.argument.argname = Some(ArgumentIdentifier::Flag(flags)),
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
        self.argument.core.choices = choices;
        self
    }
    pub fn help(mut self: Self, help: impl Into<String>) -> Self {
        self.argument.core.help = help.into();
        self
    }
}
impl ArgumentBuilder<StatePositional> {
    pub fn build(self: Self) -> PositionalOrOptional {
        PositionalOrOptional::Positional(
            self.into_positional()
        )
    }
}
impl ArgumentBuilder<StateOptional> {
    pub fn build(self: Self) -> PositionalOrOptional {
        PositionalOrOptional::Optional(
            self.into_optional()
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

#[derive(Debug, Clone)]
struct ArgumentCore {
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
    fn new_positional(name: &str) -> {
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
        let ArgumentIdentifer::Name(name) = self.argname.expect("argument identifier not set") {
            panic!("argument identifier not a name");
        };
        let core = self.core;
        PositionalArgument {
            name,
            core,
        }
    }

    fn into_optional(self: Self) -> OptionalArgument {
        let ArgumentIdentifer::Flags(flags) = self.argname.expect("argument identifier not set") {
            panic!("argument identifier not a flag(s)");
        };
        OptionalArgument {
            flags,
            default: self.default,
            required: self.required,
            core: self.core,
        }
    }
}
