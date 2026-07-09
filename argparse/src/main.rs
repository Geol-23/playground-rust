use argparse::{
    ArgumentParser,
    ArgumentAction,
    ArgumentNumber,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = ArgumentParser::builder()
        .command("find", |cmd| { cmd
            .help("Search the database for the existence of a tag meeting some criteria")
            .arg("fcs", |arg| { arg
                .flags("--fcs")
                .action(ArgumentAction::Extend)
                .nargs(ArgumentNumber::OneOrMore)
                .help("Restrict search space by fcs")
            })
            .arg("tag", |arg| { arg
                .flags("--tag")
                .action(ArgumentAction::Extend)
                .nargs(ArgumentNumber::OneOrMore)
                .help("Restrict search space by tag or glob-like expression")
            })
            .arg("block", |arg| { arg
                .flags("--block")
                .action(ArgumentAction::Extend)
                .nargs(ArgumentNumber::OneOrMore)
                .help("Restrict search space by block type")
            })
            .arg("file", |arg| { arg
                .flags("--file")
                .action(ArgumentAction::Extend)
                .nargs(ArgumentNumber::OneOrMore)
                .help("Restrict search space by file name")
            })
            .arg("type", |arg| { arg
                .flags("--type")
                .action(ArgumentAction::Extend)
                .nargs(ArgumentNumber::OneOrMore)
                .help("Restrict search space by tag type")
            })
        })
        .build();

    parser.parse()?;

    let parser = ArgumentParser::builder()
        .help("A fictional versioning cli")
        .command("clone", |cmd| { cmd
            .help("Clones repos")
            .arg("remote", |arg| { arg
                .help("The remote to clone")
            })
        })
        .command("diff", |cmd| { cmd
            .help("Compare two commits")
            .arg("base", |arg| { arg })
            .arg("head", |arg| { arg })
            .arg("path", |arg| { arg })
        })
        .build();

    // let args = parser.parse() // Option input of &[&str]
    Ok(())
}
