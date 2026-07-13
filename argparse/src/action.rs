// Standard imports
use std::fmt;
use std::rc::Rc;

// Internal imports
use super::arg_table;
use super::error::ArgParseError as ArgE;
use super::argument::{
    Argument,
    ArgumentRef,
};

pub trait Action: fmt::Debug {
    fn act(self: &Self, table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, values: &[&str]) -> Result<(), ArgE>;
}


fn store(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    match argument {
            ArgumentRef::Positional(arg) => {
                table.args.insert(
                    dest.to_string(),
                    vec![values.iter()
                        .map(|value| value.to_string())
                        .collect()]
                );
            },
            ArgumentRef::Optional(arg) => {
                if values.is_empty() {
                    match &arg.default {
                        None => { return Err(ArgE::ExpectedArguments( arg.flags.clone(), arg.core.nargs )); },
                        Some(default) => {
                            table.args.insert(
                                dest.to_string(),
                                vec![default.clone()]
                            );
                        }
                    }
                } else {
                    table.args.insert(
                        dest.to_string(),
                        vec![values.iter()
                            .map(|value| value.to_string())
                            .collect()]
                    );
                }
            },
        };
    Ok(())
}

fn store_const(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    let constant = argument.core().constant.as_ref()
        .ok_or_else(|| ArgE::UndefinedConstant(argument.label()) )?;
    table.args.insert(
        dest.to_string(),
        vec![constant.clone()]
    );
    Ok(())
}

fn store_true(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    table.args.insert(
        dest.to_string(),
        vec![vec!["true".to_string()]]
    );
    Ok(())
}

fn store_false(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    table.args.insert(
        dest.to_string(),
        vec![vec!["false".to_string()]]
    );
    Ok(())
}

fn append(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    let values: Vec<String> = values.iter().map(|s| s.to_string()).collect();
    if !values.is_empty() {
        table.args.entry(dest.to_string())
            .or_default()
            .push(values);
    }

    Ok(())
}

fn append_const(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    let constant: &Vec<String> = match &argument.core().constant {
        Some(vals) => vals,
        None => match argument {
            ArgumentRef::Positional(arg) => { return Err(ArgE::UndefinedConstant( arg.name.clone() )); },
            ArgumentRef::Optional(arg) => { return Err(ArgE::UndefinedConstant( arg.flags.join(", ") )); }
        }
    };
    table.args.entry(dest.to_string())
        .or_default()
        .push(constant.clone());
    Ok(())
}

fn extend(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    let mut values: Vec<String> = values.iter().map(|s| s.to_string()).collect();
    if !values.is_empty() {
        let groups = table.args.entry(dest.to_string())
            .or_default();
        match groups.first_mut() {
            Some(vec) => { vec.append(&mut values); },
            None => { groups.push(values); }
        };
    }

    Ok(())
}

fn count(table: &mut arg_table::ArgumentTable, dest: &str, argument: ArgumentRef, mut values: &[&str]) -> Result<(), ArgE> {
    let groups = table.args.entry(dest.to_string()).or_default();
    match groups.first_mut() {
        Some(group) => {
            group[0] = (group[0].parse::<usize>().unwrap() + 1).to_string();
        },
        None => { groups.push(vec!["1".to_string()]); },
    }
    Ok(())
}



#[derive(Debug, Clone)]
pub enum ArgumentAction {
    Store,
    StoreConst,
    StoreTrue,
    StoreFalse,
    Append,
    AppendConst,
    Extend,
    Count,
    Custom(Rc<dyn Action>),
}

impl ArgumentAction {
    pub fn act(
        self: &Self,
        table: &mut arg_table::ArgumentTable,
        dest: &str,
        argument: ArgumentRef<'_>,
        values: &[&str]
    ) -> Result<(), ArgE> {
        match self {
            ArgumentAction::Store => store(table, dest, argument, values),
            ArgumentAction::StoreConst => store_const(table, dest, argument, values),
            ArgumentAction::StoreTrue => store_true(table, dest, argument, values),
            ArgumentAction::StoreFalse => store_false(table, dest, argument, values),
            ArgumentAction::Append => append(table, dest, argument, values),
            ArgumentAction::AppendConst => append_const(table, dest, argument, values),
            ArgumentAction::Extend => extend(table, dest, argument, values),
            ArgumentAction::Count => count(table, dest, argument, values),
            ArgumentAction::Custom(action) => action.act(table, dest, argument, values),
        }
    }
}
