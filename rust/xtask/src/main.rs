use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

const DB_NAME: &str = "tinygrove-dev";
const SPACETIME_LISTEN_ADDR: &str = "127.0.0.1:3000";

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [] => {
            print_help();
            Ok(())
        }
        [cmd] if cmd == "help" || cmd == "-h" || cmd == "--help" => {
            print_help();
            Ok(())
        }
        [cmd] if cmd == "doctor" => doctor(),
        [cmd] if cmd == "check" => check(),
        [cmd] if cmd == "dev" => dev(),
        [cmd, subcmd] if cmd == "db" && subcmd == "start" => db_start(),
        [cmd, subcmd] if cmd == "db" && subcmd == "build" => db_build(),
        [cmd, subcmd] if cmd == "db" && subcmd == "publish" => db_publish(),
        [cmd, subcmd] if cmd == "db" && subcmd == "generate" => db_generate(),
        [cmd, subcmd] if cmd == "db" && subcmd == "describe" => db_describe(),
        [cmd, subcmd] if cmd == "client" && subcmd == "build" => client_build(),
        [cmd, subcmd] if cmd == "godot" && subcmd == "run" => godot_run(),
        _ => Err(format!("Unknown command: {}", args.join(" "))),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    println!(
        "\
tinygrove repo tasks

Usage:
  cargo xtask <command>

Commands:
  doctor        Check required local tools
  check         Run Rust checks for repo tooling and the SpacetimeDB module
  dev           Publish/generate, then launch Godot
  db start      Start a local SpacetimeDB server on {SPACETIME_LISTEN_ADDR}
  db build      Build the SpacetimeDB module
  db publish    Publish the module to local database {DB_NAME}
  db generate   Generate Rust client bindings into rust/client/generated
  db describe   Describe the local database schema
  client build  Build and stage the Godot Rust extension
  godot run     Launch the Godot project
"
    );
}

fn doctor() -> Result<(), String> {
    check_tool("cargo", ["--version"])?;
    check_tool("spacetime", ["--version"])?;
    check_tool("godot", ["--version"])?;
    println!("doctor: all required tools are available");
    Ok(())
}

fn check() -> Result<(), String> {
    run("cargo", ["check", "--workspace"], rust_dir())
}

fn dev() -> Result<(), String> {
    db_publish()?;
    db_generate()?;
    client_build()?;
    godot_run()
}

fn db_start() -> Result<(), String> {
    exec(
        "spacetime",
        [
            "start",
            "--listen-addr",
            SPACETIME_LISTEN_ADDR,
            "--data-dir",
            ".spacetime-data",
        ],
        repo_dir(),
    )
}

fn db_build() -> Result<(), String> {
    run(
        "spacetime",
        ["build", "--module-path", "rust/server"],
        repo_dir(),
    )
}

fn db_publish() -> Result<(), String> {
    run(
        "spacetime",
        [
            "publish",
            DB_NAME,
            "--server",
            "local",
            "--module-path",
            "rust/server",
            "--yes",
        ],
        repo_dir(),
    )
}

fn db_generate() -> Result<(), String> {
    run(
        "spacetime",
        [
            "generate",
            DB_NAME,
            "--lang",
            "rust",
            "--out-dir",
            "rust/client/generated",
            "--module-path",
            "rust/server",
            "--yes",
        ],
        repo_dir(),
    )
}

fn db_describe() -> Result<(), String> {
    run(
        "spacetime",
        ["describe", DB_NAME, "--server", "local", "--json"],
        repo_dir(),
    )
}

fn client_build() -> Result<(), String> {
    run(
        "cargo",
        ["build", "--package", "tinygrove_client"],
        rust_dir(),
    )?;
    let source = client_library_path();
    let target = repo_dir().join("godot").join("bin").join(
        source
            .file_name()
            .ok_or("client library path has no file name")?,
    );
    std::fs::create_dir_all(
        target
            .parent()
            .ok_or("client library target has no parent")?,
    )
    .map_err(|error| format!("failed to create Godot bin directory: {error}"))?;
    std::fs::copy(&source, &target).map_err(|error| {
        format!(
            "failed to copy {} to {}: {error}",
            source.display(),
            target.display()
        )
    })?;
    println!("staged {}", target.display());
    Ok(())
}

fn godot_run() -> Result<(), String> {
    exec("godot", ["--path", "godot"], repo_dir())
}

fn check_tool<const N: usize>(program: &str, args: [&str; N]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| format!("failed to run `{program}`: {error}"))?;

    if !output.status.success() {
        return Err(format!("`{program}` exited with {}", output.status));
    }

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("ok");
    println!("{program}: {first_line}");
    Ok(())
}

fn run<I, S>(program: &str, args: I, cwd: PathBuf) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|error| format!("failed to run `{program}`: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program}` exited with {status}"))
    }
}

fn exec<I, S>(program: &str, args: I, cwd: PathBuf) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let error = Command::new(program).args(args).current_dir(cwd).exec();
    Err(format!("failed to exec `{program}`: {error}"))
}

#[cfg(unix)]
trait CommandExec {
    fn exec(&mut self) -> std::io::Error;
}

#[cfg(unix)]
impl CommandExec for Command {
    fn exec(&mut self) -> std::io::Error {
        use std::os::unix::process::CommandExt;
        CommandExt::exec(self)
    }
}

#[cfg(not(unix))]
trait CommandExec {
    fn exec(&mut self) -> std::io::Error;
}

#[cfg(not(unix))]
impl CommandExec for Command {
    fn exec(&mut self) -> std::io::Error {
        match self.status() {
            Ok(status) if status.success() => {
                std::io::Error::new(std::io::ErrorKind::Other, "process exited")
            }
            Ok(status) => std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("process exited with {status}"),
            ),
            Err(error) => error,
        }
    }
}

fn repo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask should live at rust/xtask")
        .to_path_buf()
}

fn rust_dir() -> PathBuf {
    repo_dir().join("rust")
}

fn client_library_path() -> PathBuf {
    let file_name = if cfg!(target_os = "macos") {
        "libtinygrove_client.dylib"
    } else if cfg!(target_os = "windows") {
        "tinygrove_client.dll"
    } else {
        "libtinygrove_client.so"
    };

    rust_dir().join("target").join("debug").join(file_name)
}
