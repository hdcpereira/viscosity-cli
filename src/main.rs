use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::process::Command;

const LIST_CONNECTIONS_SCRIPT: &str = r#"
tell application "Viscosity"
	set rows to {}
	repeat with c in (every connection)
		set end of rows to (name of c & tab & state of c)
	end repeat
	set oldDelims to AppleScript's text item delimiters
	set AppleScript's text item delimiters to linefeed
	set outText to rows as string
	set AppleScript's text item delimiters to oldDelims
	return outText
end tell
"#;

#[derive(Parser)]
#[command(
    name = "viscosity-cli",
    version,
    about = "Control Viscosity VPN from the terminal (macOS, via osascript)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a connection (1-based index from `list`, or full name)
    Connect {
        /// Index (digits only, e.g. 3) or connection name as shown in Viscosity / `list`
        target: String,
    },
    /// Disconnect from a connection (1-based index from `list`, or full name)
    Disconnect {
        /// Index (digits only) or connection name
        target: String,
    },
    /// List connections in a table (with # for use with connect/disconnect)
    List,
}

fn osascript(script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .context("failed to spawn `osascript`; is this running on macOS?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("osascript failed (status {}):\n{}", output.status, err.trim_end());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn osascript_with_argv(script: &str, argv: &[&str]) -> Result<String> {
    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(script);
    if !argv.is_empty() {
        cmd.arg("--");
        for arg in argv {
            cmd.arg(arg);
        }
    }

    let output = cmd
        .output()
        .context("failed to spawn `osascript`; is this running on macOS?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("osascript failed (status {}):\n{}", output.status, err.trim_end());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_connection_tsv(tsv: &str) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        match line.split_once('\t') {
            Some((name, state)) => rows.push((name.to_string(), state.to_string())),
            None => rows.push((line.to_string(), String::new())),
        }
    }
    rows
}

fn fetch_connection_rows() -> Result<Vec<(String, String)>> {
    let out = osascript(LIST_CONNECTIONS_SCRIPT.trim())?;
    Ok(parse_connection_tsv(&out))
}

fn looks_like_index(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn resolve_connection_target(arg: &str) -> Result<String> {
    let t = arg.trim();
    if t.is_empty() {
        anyhow::bail!("connection target is empty");
    }

    if looks_like_index(t) {
        let n: usize = t
            .parse()
            .context("index should be a positive integer (see `viscosity-cli list`)")?;
        if n == 0 {
            anyhow::bail!("connection index must be >= 1 (see `viscosity-cli list`)");
        }
        let rows = fetch_connection_rows()?;
        if rows.is_empty() {
            anyhow::bail!("no connections returned by Viscosity");
        }
        let name = rows
            .get(n - 1)
            .map(|(name, _)| name.clone())
            .with_context(|| {
                format!(
                    "connection index {} is out of range (valid: 1–{})",
                    n,
                    rows.len()
                )
            })?;
        return Ok(name);
    }

    Ok(t.to_string())
}

fn connect(connection_name: &str) -> Result<()> {
    const SCRIPT: &str = r#"
on run argv
	tell application "Viscosity" to connect (item 1 of argv as string)
end run
"#;
    osascript_with_argv(SCRIPT.trim(), &[connection_name])?;
    Ok(())
}

fn disconnect(connection_name: &str) -> Result<()> {
    const SCRIPT: &str = r#"
on run argv
	tell application "Viscosity" to disconnect (item 1 of argv as string)
end run
"#;
    osascript_with_argv(SCRIPT.trim(), &[connection_name])?;
    Ok(())
}

fn list() -> Result<()> {
    let rows = fetch_connection_rows()?;
    print_connection_table_from_rows(&rows);
    Ok(())
}

fn print_connection_table_from_rows(rows: &[(String, String)]) {
    if rows.is_empty() {
        return;
    }

    const H_NUM: &str = "#";
    const H_NAME: &str = "Connection";
    const H_STATE: &str = "State";

    let num_w = rows
        .len()
        .to_string()
        .chars()
        .count()
        .max(H_NUM.chars().count());
    let name_w = rows
        .iter()
        .map(|(n, _)| n.chars().count())
        .max()
        .unwrap_or(0)
        .max(H_NAME.chars().count());
    let state_w = rows
        .iter()
        .map(|(_, s)| s.chars().count())
        .max()
        .unwrap_or(0)
        .max(H_STATE.chars().count());

    let rule = |left: &str, m1: &str, m2: &str, right: &str| {
        format!(
            "{}{}{}{}{}{}{}{}",
            left,
            "─".repeat(num_w + 2),
            m1,
            "─".repeat(name_w + 2),
            m2,
            "─".repeat(state_w + 2),
            right,
            "\n"
        )
    };

    print!("{}", rule("┌", "┬", "┬", "┐"));
    println!(
        "│ {:>nw$} │ {:<name_w$} │ {:<sw$} │",
        H_NUM,
        H_NAME,
        H_STATE,
        nw = num_w,
        name_w = name_w,
        sw = state_w
    );
    print!("{}", rule("├", "┼", "┼", "┤"));
    for (i, (name, state)) in rows.iter().enumerate() {
        println!(
            "│ {:>nw$} │ {:<name_w$} │ {:<sw$} │",
            i + 1,
            name,
            state,
            nw = num_w,
            name_w = name_w,
            sw = state_w
        );
    }
    print!("{}", rule("└", "┴", "┴", "┘"));
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Connect { target } => {
            let name = resolve_connection_target(&target)?;
            connect(&name)?;
        }
        Commands::Disconnect { target } => {
            let name = resolve_connection_target(&target)?;
            disconnect(&name)?;
        }
        Commands::List => list()?,
    }

    Ok(())
}
