// Standard imports
use std::fmt;

// Internal imports
use super::arg_table;
use super::error::ArgParseError as ArgE;
use super::argument::PositionalOrOptional as PoO;

pub trait Action: fmt::Debug {
    fn act(self: &Self, table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, values: &[&str]) -> Result<(), ArgE>;
}


fn store(table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
    match argument {
        PoO::Positional(arg) => {
            table.args.insert(
                dest.to_string(),
                vec![values.iter()
                    .map(|value| value.to_string())
                    .collect()]
            );
        },
        PoO::Optional(arg) => {
            if values.is_empty() {
                match &arg.default {
                    None => { return Err(ArgE::ExpectedArguments( arg.clone() )); },
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

fn store_const(table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
    let constant = match argument {
        PoO::Positional(arg) => {
            if let Some(constant) = &arg.constant {
                constant
            } else {
                return Err(ArgE::UndefinedConstant( arg.name.clone() ));
            }
        },
        PoO::Optional(arg) => {
            if let Some(constant) = &arg.constant {
                constant
            } else {
                return Err(ArgE::UndefinedConstant( arg.flags.join(", ") ));
            }
        },
    };
    table.args.insert(
        dest.to_string(),
        vec![constant.iter()
            .map(|val| val.to_string())
            .collect()]
    );
    Ok(())
}

fn store_true(table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
    table.args.insert(
        dest.to_string(),
        vec![vec!["true".to_string()]]
    );
    Ok(())
}

fn store_false(table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
    table.args.insert(
        dest.to_string(),
        vec![vec!["false".to_string()]]
    );
    Ok(())
}

fn append(self: &Self, table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
    let values: Vec<String> = values.iter().map(|s| s.to_string()).collect();
    if !values.is_empty() {
        table.args.entry(dest.to_string())
            .or_default()
            .push(values);
    }

    Ok(())
}

fn append_const(self: &Self, table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
    let constant = match argument {
        PoO::Positional(arg) => {
            if let Some(constant) = &arg.constant {
                constant
            } else {
                return Err(ArgE::UndefinedConstant( arg.name.clone() ));
            }
        },
        PoO::Optional(arg) => {
            if let Some(constant) = &arg.constant {
                constant
            } else {
                return Err(ArgE::UndefinedConstant( arg.flags.join(", ") ));
            }
        },
    };
    table.args.entry(dest.to_string())
        .or_default()
        .push(
            constant.iter()
                .map(|val| val.to_string())
                .collect()
        );
    Ok(())
}

fn extend(self: &Self, table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
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

fn count(self: &Self, table: &mut arg_table::ArgumentTable, dest: &str, argument: &PoO, mut values: &[&str]) -> Result<(), ArgE> {
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
    fn act(
        self: &Self,
        table: &mut arg_table::ArgumentTable,
        dest: &str,
        argument: &PoO,
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
