use std::{env, str};
use std::io::{self, Write, Read};

use std::process::{Command, Stdio};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, Error>;

fn get_expanded_target() -> crate::Result<Vec<u8>> {
    let cargo_path = env::var("CARGO")?;

    let mut args = env::args_os().skip(2);

    let cargo_args = args.by_ref().take_while(|arg| arg != "--").collect::<Vec<_>>();

    let rustc_args = args.collect::<Vec<_>>();

    let mut command = Command::new(cargo_path);

    command.arg("rustc")
        .args(cargo_args)
        .arg("--")
        .arg("-Z")
        .arg("unstable-options")
        .arg("--pretty=expanded")
        .arg("--cfg=__sqlx_gather_queries")
        .args(rustc_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped());

    println!("cargo command: {:?}", command);

    let mut child = command.spawn()?;

    let mut stdout = Vec::new();
    child.stdout.as_mut().unwrap().read_to_end(&mut stdout);

    if !child.wait()?.success() {
        return Err("cargo rustc completed unsuccessfully".into());
    }

    Ok(stdout)
}

fn collect_sql_strings(mut input: &str) -> Result<Vec<&str>> {
    let mut strings = Vec::new();

    while let Some((string, rem)) = find_next_sql_string(input)? {
        strings.push(string);
        input = rem;
    }

    Ok(strings)
}

fn find_next_sql_string(input: &str) -> Result<Option<(&str, &str)>> {
    const STRING_START: &str = "__sqlx_checked_sql_noop(\"";
    const STRING_END: &str = "\")";

    if let Some(idx) = input.find(STRING_START) {
        let start = idx + STRING_START.len();

        while let Some(end) = input[start..].find(STRING_END) {
            if &input[start + end - 1 .. start + end] != "\\" {
                return Ok(Some(input[start..].split_at(end)));
            }
        }

        return Err(format!("unterminated SQL: {}", &input[start..]).into());
    }

    return Ok(None);
}

fn main() -> Result<()> {
    let expanded = get_expanded_target()?;
    let sql_strings = collect_sql_strings(str::from_utf8(&expanded)?)?;

    println!("{}", sql_strings.join("\n"));

    Ok(())
}
