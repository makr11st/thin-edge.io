mod module_check;
use std::process::{Command, ExitStatus, Stdio};
use structopt::StructOpt;

#[derive(StructOpt)]
struct AptCli {
    #[structopt(subcommand)]
    operation: PluginOp,
}

#[derive(StructOpt)]
pub enum PluginOp {
    /// List all the installed modules
    List,

    /// Install a module
    Install {
        module: String,
        #[structopt(short = "v", long = "--module-version")]
        version: Option<String>,
        #[structopt(long = "--file")]
        file_path: Option<String>,
    },

    /// Uninstall a module
    Remove {
        module: String,
        #[structopt(short = "v", long = "--module-version")]
        version: Option<String>,
    },

    /// Prepare a sequences of install/remove commands
    Prepare,

    /// Finalize a sequences of install/remove commands
    Finalize,
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("Fail to run `{cmd}`: {from}")]
    ExecError { cmd: String, from: std::io::Error },
}

impl InternalError {
    pub fn exec_error(cmd: impl Into<String>, from: std::io::Error) -> InternalError {
        InternalError::ExecError {
            cmd: cmd.into(),
            from,
        }
    }
}

fn run(operation: PluginOp) -> Result<ExitStatus, InternalError> {
    let status = match operation {
        PluginOp::List {} => {
            let apt = Command::new("apt")
                .args(vec!["--manual-installed", "list"])
                .stdout(Stdio::piped()) // To pipe apt.stdout into awk.stdin
                .spawn()
                .map_err(|err| InternalError::exec_error("apt", err))?;

            // apt output    = openssl/focal-security,now 1.1.1f-1ubuntu2.3 amd64 [installed]
            // awk -F '[/ ]' =   $1   ^       $2         ^   $3            ^   $4
            // awk print     =   name ^                  ^   version       ^
            let status = Command::new("awk")
                .args(vec![
                    "-F",
                    "[/ ]",
                    r#"{if ($1 != "Listing...") { print "{\"name\":\""$1"\",\"version\":\""$3"\"}"}}"#,
                ])
                .stdin(apt.stdout.unwrap()) // Cannot panic: apt.stdout has been set
                .status()
                .map_err(|err| InternalError::exec_error("awk", err))?;

            status
        }

        PluginOp::Install {
            module,
            version,
            file_path,
        } => {
            // NOTE: I don't like this logic, i think it can be improved.
            if let Some(version) = version {
                // check if we also have file_path
                if let Some(file_path) = file_path {
                    dbg!("fp and version provided");

                    dbg!(module_check::module_has_extension(&file_path));

                    let pm = module_check::PackageMetadata::new()
                        .try_new(&file_path.as_str())
                        .unwrap();

                    dbg!(pm.metadata_contains(&format!("Version: {}", &version)));
                    dbg!(pm.metadata_contains(&format!("Package: {}", &module)));

                    run_cmd("apt-get", &format!("install --quiet --yes {}", file_path))?
                } else {
                    // only module version provided
                    run_cmd(
                        "apt-get",
                        &format!("install --quiet --yes {}={}", module, version),
                    )?
                }
            } else if let Some(file_path) = file_path {
                run_cmd("apt-get", &format!("install --quiet --yes {}", file_path))?
            } else {
                run_cmd("apt-get", &format!("install --quiet --yes {}", module))?
            }
        }

        PluginOp::Remove { module, version } => {
            if let Some(version) = version {
                // check the version mentioned present or not
                run_cmd(
                    "apt-get",
                    &format!("remove --quiet --yes {}={}", module, version),
                )?
            } else {
                run_cmd("apt-get", &format!("remove --quiet --yes {}", module))?
            }
        }

        PluginOp::Prepare => run_cmd("apt-get", "update --quiet --yes")?,

        PluginOp::Finalize => run_cmd("apt-get", "auto-remove --quiet --yes")?,
    };

    Ok(status)
}

fn run_cmd(cmd: &str, args: &str) -> Result<ExitStatus, InternalError> {
    let args: Vec<&str> = args.split_whitespace().collect();
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .status()
        .map_err(|err| InternalError::exec_error(cmd, err))?;
    Ok(status)
}

fn main() {
    // On usage error, the process exits with a status code of 1
    let apt = AptCli::from_args();

    match run(apt.operation) {
        Ok(status) if status.success() => {
            std::process::exit(0);
        }

        Ok(status) => {
            if status.code().is_some() {
                std::process::exit(2);
            } else {
                eprintln!("Interrupted by a signal!");
                std::process::exit(4);
            }
        }

        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(5);
        }
    }
}
