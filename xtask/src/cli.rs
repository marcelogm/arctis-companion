use std::path::PathBuf;

const USAGE: &str = concat!(
    "usage: cargo xtask <command> [--root DIR]\n",
    "  check-rules [--staged]\n",
    "  check-commits [--message FILE] [--range RANGE] [--pr-title TEXT] [--pr-body TEXT]\n",
    "  check-doc-paths\n",
    "  check-pure\n",
    "  install-hooks",
);

pub enum Invocation {
    CheckRules {
        root: PathBuf,
        staged: bool,
    },
    CheckCommits {
        root: PathBuf,
        sources: Vec<MessageSource>,
    },
    CheckDocPaths {
        root: PathBuf,
    },
    CheckPure {
        root: PathBuf,
    },
    InstallHooks {
        root: PathBuf,
    },
}

pub enum MessageSource {
    File(PathBuf),
    Range(String),
    PrTitle(String),
    PrBody(String),
}

struct Options {
    root: PathBuf,
    staged: bool,
    sources: Vec<MessageSource>,
}

pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Invocation, String> {
    let command = args.next().ok_or(USAGE)?;
    let accepted = accepted_flags(&command).ok_or_else(|| unknown_command(&command))?;
    let options = parse_options(&command, accepted, args)?;
    build(&command, options)
}

fn accepted_flags(command: &str) -> Option<&'static [&'static str]> {
    match command {
        "check-rules" => Some(&["--root", "--staged"]),
        "check-commits" => Some(&["--root", "--message", "--range", "--pr-title", "--pr-body"]),
        "check-doc-paths" | "check-pure" | "install-hooks" => Some(&["--root"]),
        _ => None,
    }
}

fn parse_options(
    command: &str,
    accepted: &[&str],
    mut args: impl Iterator<Item = String>,
) -> Result<Options, String> {
    let mut options = Options {
        root: default_root(),
        staged: false,
        sources: Vec::new(),
    };
    while let Some(flag) = args.next() {
        if !accepted.contains(&flag.as_str()) {
            return Err(format!("unknown flag {flag} for {command}\n{USAGE}"));
        }
        apply_flag(&mut options, &flag, &mut args)?;
    }
    Ok(options)
}

fn apply_flag(
    options: &mut Options,
    flag: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), String> {
    if flag == "--staged" {
        options.staged = true;
        return Ok(());
    }
    let value = args
        .next()
        .ok_or_else(|| format!("{flag} needs a value\n{USAGE}"))?;
    match flag {
        "--root" => options.root = PathBuf::from(value),
        "--message" => options
            .sources
            .push(MessageSource::File(PathBuf::from(value))),
        "--range" => options.sources.push(MessageSource::Range(value)),
        "--pr-title" => options.sources.push(MessageSource::PrTitle(value)),
        _ => options.sources.push(MessageSource::PrBody(value)),
    }
    Ok(())
}

fn build(command: &str, options: Options) -> Result<Invocation, String> {
    let Options {
        root,
        staged,
        sources,
    } = options;
    match command {
        "check-rules" => Ok(Invocation::CheckRules { root, staged }),
        "check-commits" => Ok(Invocation::CheckCommits { root, sources }),
        "check-doc-paths" => Ok(Invocation::CheckDocPaths { root }),
        "check-pure" => Ok(Invocation::CheckPure { root }),
        "install-hooks" => Ok(Invocation::InstallHooks { root }),
        _ => Err(unknown_command(command)),
    }
}

fn unknown_command(command: &str) -> String {
    format!("unknown command {command}\n{USAGE}")
}

fn default_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}
