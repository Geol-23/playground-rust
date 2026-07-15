// tests/integration_test.rs
use argparse::{
    ArgumentParser,
    ArgumentAction,
    ArgumentNumber,
};

#[test]
fn test_1_basic() -> Result<(), Box<dyn std::error::Error>>  {
    let parser = ArgumentParser::builder()
    .command("find", |cmd| { cmd
        .help("Search the database for the existence of a tag meeting some criteria")
        .arg("fcs", |arg| { arg
            .flags("--fcs")
            .action(ArgumentAction::Extend)
            .nargs(ArgumentNumber::OneOrMore)
            .default(Vec::<String>::new())
            .help("Restrict search space by fcs")
        })
        .arg("tag", |arg| { arg
            .flags("--tag")
            .action(ArgumentAction::Extend)
            .nargs(ArgumentNumber::OneOrMore)
            .default(Vec::<String>::new())
            .help("Restrict search space by tag or glob-like expression")
        })
        .arg("block", |arg| { arg
            .flags("--block")
            .action(ArgumentAction::Extend)
            .nargs(ArgumentNumber::OneOrMore)
            .default(Vec::<String>::new())
            .help("Restrict search space by block type")
        })
        .arg("file", |arg| { arg
            .flags("--file")
            .action(ArgumentAction::Extend)
            .nargs(ArgumentNumber::OneOrMore)
            .default(Vec::<String>::new())
            .help("Restrict search space by file name")
        })
        .arg("type", |arg| { arg
            .flags("--type")
            .action(ArgumentAction::Extend)
            .nargs(ArgumentNumber::OneOrMore)
            .default(Vec::<String>::new())
            .help("Restrict search space by tag type")
        })
    })
    .build();
    let args = vec!["find", "--fcs", "FCS0209", "--type", "SIO-11"];
    let table = parser.parse_from(&args)?;
    dbg!(table);
    Ok(())
}
