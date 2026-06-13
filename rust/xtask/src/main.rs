use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, Stdio},
    thread,
    time::Duration,
};

const DB_NAME: &str = "tinygrove-dev";
const SPACETIME_LISTEN_ADDR: &str = "127.0.0.1:3000";
const TEST_DB_NAME: &str = "tinygrove-test";
const TEST_SPACETIME_LISTEN_ADDR: &str = "0.0.0.0:3000";
const TEST_CONFIRM_FLAG: &str = "--confirm-durable-test";
const DEFAULT_AGENT_NAME: &str = "Agent";
const DEFAULT_AGENT_PROFILE: &str = "agent";
const DEFAULT_AGENT_SERVER_URI: &str = "http://127.0.0.1:3000";

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
        [cmd, subcmd] if cmd == "test-server" && subcmd == "start" => test_server_start(),
        [cmd, subcmd] if cmd == "test-server" && subcmd == "status" => test_server_status(),
        [cmd, subcmd] if cmd == "test-server" && subcmd == "describe" => test_server_describe(),
        [cmd, subcmd, flag] if cmd == "test-server" && subcmd == "publish" => {
            test_server_publish(flag)
        }
        [cmd, subcmd] if cmd == "test-server" && subcmd == "publish" => Err(format!(
            "durable test publish requires {TEST_CONFIRM_FLAG}; this command preserves existing data and should be intentional"
        )),
        [cmd, subcmd] if cmd == "agent" && subcmd == "list" => agent_list(),
        [cmd, subcmd, rest @ ..] if cmd == "agent" && subcmd == "run" => agent_run(rest),
        [cmd, subcmd, rest @ ..] if cmd == "agent" && subcmd == "spawn" => agent_spawn(rest),
        [cmd, subcmd] if cmd == "client" && subcmd == "build" => client_build(),
        [cmd, subcmd] if cmd == "godot" && subcmd == "run" => godot_run(),
        [cmd, subcmd] if cmd == "smoke" && subcmd == "two-clients" => smoke_two_clients(),
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
  test-server start
                Start a durable SpacetimeDB server on {TEST_SPACETIME_LISTEN_ADDR}
  test-server status
                Check whether the durable test database {TEST_DB_NAME} is reachable
  test-server describe
                Describe the durable test database schema
  test-server publish {TEST_CONFIRM_FLAG}
                Publish to durable test database {TEST_DB_NAME} without deleting data
  agent run [--name NAME] [--profile PROFILE] [--server-uri URI] [--database NAME] [--port PORT]
                Launch an agent Godot client in the foreground, defaulting to {TEST_DB_NAME}
  agent spawn [--name NAME] [--profile PROFILE] [--server-uri URI] [--database NAME] [--port PORT]
                Launch an agent Godot client in the background and print its registry entry
  agent list    List Tiny Grove loopback registry entries and stream URLs
  client build  Build and stage the Godot Rust extension
  godot run     Launch the Godot project
  smoke two-clients
                Run two headless Godot clients and verify replicated DB rows
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
            "--delete-data=always",
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

fn test_server_start() -> Result<(), String> {
    exec(
        "spacetime",
        [
            "start",
            "--listen-addr",
            TEST_SPACETIME_LISTEN_ADDR,
            "--data-dir",
            ".spacetime-test-data",
        ],
        repo_dir(),
    )
}

fn test_server_status() -> Result<(), String> {
    println!("durable test server target:");
    println!("  listen addr: {TEST_SPACETIME_LISTEN_ADDR}");
    println!("  database:    {TEST_DB_NAME}");
    println!("  data dir:    .spacetime-test-data");
    run(
        "spacetime",
        [
            "describe",
            TEST_DB_NAME,
            "--server",
            "local",
            "--json",
            "--no-config",
        ],
        repo_dir(),
    )
}

fn test_server_describe() -> Result<(), String> {
    run(
        "spacetime",
        [
            "describe",
            TEST_DB_NAME,
            "--server",
            "local",
            "--json",
            "--no-config",
        ],
        repo_dir(),
    )
}

fn test_server_publish(flag: &str) -> Result<(), String> {
    if flag != TEST_CONFIRM_FLAG {
        return Err(format!(
            "durable test publish requires exactly {TEST_CONFIRM_FLAG}, got `{flag}`"
        ));
    }

    println!("publishing durable test server update");
    println!("  database:    {TEST_DB_NAME}");
    println!("  server:      local SpacetimeDB at {TEST_SPACETIME_LISTEN_ADDR}");
    println!("  data policy: preserve existing data");
    println!("  destructive publish: disabled");
    check()?;
    run(
        "spacetime",
        [
            "publish",
            TEST_DB_NAME,
            "--server",
            "local",
            "--module-path",
            "rust/server",
            "--no-config",
            "--yes",
        ],
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
    sign_client_library(&target)?;
    println!("staged {}", target.display());
    Ok(())
}

fn sign_client_library(target: &Path) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }

    run(
        "codesign",
        [
            "--force",
            "--sign",
            "-",
            target
                .to_str()
                .ok_or("client library target path is not valid UTF-8")?,
        ],
        repo_dir(),
    )
}

fn godot_run() -> Result<(), String> {
    exec("godot", ["--path", "godot"], repo_dir())
}

#[derive(Debug)]
struct AgentOptions {
    name: String,
    profile: String,
    server_uri: String,
    database_name: String,
    port: Option<String>,
}

fn agent_run(args: &[String]) -> Result<(), String> {
    let options = AgentOptions::parse(args)?;
    print_agent_launch_summary(&options, false);
    exec_agent_godot(&options)
}

fn agent_spawn(args: &[String]) -> Result<(), String> {
    let options = AgentOptions::parse(args)?;
    fs::create_dir_all(agent_log_dir())
        .map_err(|error| format!("failed to create agent log directory: {error}"))?;
    let log_path = agent_log_path(&options);
    let log = fs::File::create(&log_path)
        .map_err(|error| format!("failed to create {}: {error}", log_path.display()))?;
    let err_log = log
        .try_clone()
        .map_err(|error| format!("failed to clone {}: {error}", log_path.display()))?;

    print_agent_launch_summary(&options, true);
    let mut command = agent_command(&options);
    command.stdout(Stdio::from(log));
    command.stderr(Stdio::from(err_log));
    let child = command
        .spawn()
        .map_err(|error| format!("failed to spawn Godot agent client: {error}"))?;

    println!("  pid:       {}", child.id());
    println!("  log:       {}", log_path.display());
    println!("  registry:  {}", agent_registry_dir().display());
    wait_for_agent_registry(&options)
}

fn agent_list() -> Result<(), String> {
    let entries = read_agent_registry_entries()?;
    if entries.is_empty() {
        println!(
            "no Tiny Grove agent registry entries found in {}",
            agent_registry_dir().display()
        );
        return Ok(());
    }

    println!("Tiny Grove agent registry entries:");
    for entry in entries {
        let profile = json_str(&entry, "profile").unwrap_or("?");
        let display_name = json_str(&entry, "display_name").unwrap_or("");
        let agent_name = json_str(&entry, "agent_name").unwrap_or("");
        let name = if !display_name.is_empty() {
            display_name
        } else if !agent_name.is_empty() {
            agent_name
        } else {
            "?"
        };
        let db = json_str(&entry, "database_name").unwrap_or("?");
        let server = json_str(&entry, "server_uri").unwrap_or("?");
        let base_url = json_str(&entry, "base_url").unwrap_or("?");
        let stream_url = stream_url_for(&entry);
        let stream_name = watch_stream_name_for(&entry);
        let connected = entry
            .get("connected")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        println!("- {name} profile={profile} connected={connected} db={db} server={server}");
        println!("  base_url:        {base_url}");
        println!(
            "  watch stream:    {}",
            stream_name.unwrap_or_else(|| "?".to_string())
        );
        println!(
            "  stream_url:      {}",
            stream_url.unwrap_or_else(|| "?".to_string())
        );
    }
    Ok(())
}

impl AgentOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            name: DEFAULT_AGENT_NAME.to_string(),
            profile: DEFAULT_AGENT_PROFILE.to_string(),
            server_uri: DEFAULT_AGENT_SERVER_URI.to_string(),
            database_name: TEST_DB_NAME.to_string(),
            port: None,
        };

        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--name" => {
                    options.name = value_arg(args, &mut index, "--name")?;
                }
                "--profile" => {
                    options.profile = value_arg(args, &mut index, "--profile")?;
                }
                "--server-uri" => {
                    options.server_uri = value_arg(args, &mut index, "--server-uri")?;
                }
                "--database" | "--db" => {
                    let flag = args[index].clone();
                    options.database_name = value_arg(args, &mut index, &flag)?;
                }
                "--port" => {
                    options.port = Some(value_arg(args, &mut index, "--port")?);
                }
                "--local-dev" => {
                    options.database_name = DB_NAME.to_string();
                }
                "--test-server" => {
                    options.database_name = TEST_DB_NAME.to_string();
                }
                "--help" | "-h" => {
                    return Err(agent_usage());
                }
                other => {
                    return Err(format!(
                        "unknown agent option `{other}`\n\n{}",
                        agent_usage()
                    ));
                }
            }
            index += 1;
        }

        Ok(options)
    }
}

fn value_arg(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    let value = args
        .get(*index + 1)
        .ok_or_else(|| format!("{flag} requires a value"))?
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(format!("{flag} requires a non-empty value"));
    }
    *index += 1;
    Ok(value)
}

fn agent_usage() -> String {
    format!(
        "Usage: cargo xtask agent run|spawn [--name NAME] [--profile PROFILE] [--server-uri URI] [--database NAME] [--port PORT]\nDefaults: --name {DEFAULT_AGENT_NAME} --profile {DEFAULT_AGENT_PROFILE} --server-uri {DEFAULT_AGENT_SERVER_URI} --database {TEST_DB_NAME}"
    )
}

fn print_agent_launch_summary(options: &AgentOptions, background: bool) {
    println!("launching Tiny Grove agent client");
    println!("  name:      {}", options.name);
    println!("  profile:   {}", options.profile);
    println!("  server:    {}", options.server_uri);
    println!("  database:  {}", options.database_name);
    println!(
        "  port:      {}",
        options.port.as_deref().unwrap_or("auto-scan from 37373")
    );
    println!(
        "  mode:      {}",
        if background {
            "background"
        } else {
            "foreground"
        }
    );
}

fn agent_command(options: &AgentOptions) -> Command {
    let mut command = Command::new("godot");
    command
        .arg("--path")
        .arg("godot")
        .current_dir(repo_dir())
        .env("TINYGROVE_AGENT_PROFILE", &options.profile)
        .env("TINYGROVE_AGENT_NAME", &options.name)
        .env("TINYGROVE_SERVER_URI", &options.server_uri)
        .env("TINYGROVE_DATABASE_NAME", &options.database_name);
    if let Some(port) = &options.port {
        command.env("TINYGROVE_AGENT_PORT", port);
    }
    command
}

fn exec_agent_godot(options: &AgentOptions) -> Result<(), String> {
    let error = agent_command(options).exec();
    Err(format!("failed to exec Godot agent client: {error}"))
}

fn wait_for_agent_registry(options: &AgentOptions) -> Result<(), String> {
    for _ in 0..30 {
        if let Some(entry) = find_agent_registry_entry(options)? {
            let base_url = json_str(&entry, "base_url").unwrap_or("?");
            let stream_url = stream_url_for(&entry).unwrap_or_else(|| "?".to_string());
            let stream_name = watch_stream_name_for(&entry).unwrap_or_else(|| "?".to_string());
            let connected = entry
                .get("connected")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            println!("  base_url:  {base_url}");
            println!("  stream:    {stream_name}");
            println!("  sse url:   {stream_url}");
            println!("  connected: {connected}");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    println!(
        "agent client was spawned, but no matching registry entry appeared yet; run `cargo xtask agent list` after Godot finishes booting"
    );
    Ok(())
}

fn find_agent_registry_entry(options: &AgentOptions) -> Result<Option<serde_json::Value>, String> {
    let mut matches = read_agent_registry_entries()?
        .into_iter()
        .filter(|entry| json_str(entry, "profile") == Some(options.profile.as_str()))
        .filter(|entry| json_str(entry, "database_name") == Some(options.database_name.as_str()))
        .filter(|entry| {
            json_str(entry, "agent_name") == Some(options.name.as_str())
                || json_str(entry, "display_name") == Some(options.name.as_str())
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        let left_time = left
            .get("updated_unix")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let right_time = right
            .get("updated_unix")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        right_time
            .partial_cmp(&left_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(matches.into_iter().next())
}

fn read_agent_registry_entries() -> Result<Vec<serde_json::Value>, String> {
    let dir = agent_registry_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for item in
        fs::read_dir(&dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?
    {
        let path = item
            .map_err(|error| format!("failed to read {} entry: {error}", dir.display()))?
            .path();
        if path.extension().and_then(OsStr::to_str) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => entries.push(value),
            Err(error) => eprintln!("warning: skipped {}: {error}", path.display()),
        }
    }
    Ok(entries)
}

fn json_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(serde_json::Value::as_str)
}

fn stream_url_for(value: &serde_json::Value) -> Option<String> {
    json_str(value, "stream_url")
        .map(str::to_string)
        .or_else(|| {
            json_str(value, "base_url").map(|base_url| format!("{base_url}/stream?waking=true"))
        })
}

fn watch_stream_name_for(value: &serde_json::Value) -> Option<String> {
    json_str(value, "watch_stream_name")
        .map(str::to_string)
        .or_else(|| {
            let profile = json_str(value, "profile")?;
            let port = value.get("port")?.as_i64()?;
            Some(format!("tinygrove:{profile}:{port}"))
        })
}

fn agent_registry_dir() -> PathBuf {
    repo_dir().join(".tinygrove").join("agents")
}

fn agent_log_dir() -> PathBuf {
    repo_dir().join(".tinygrove").join("logs")
}

fn agent_log_path(options: &AgentOptions) -> PathBuf {
    let safe_name = options
        .name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    agent_log_dir().join(format!("agent-{safe_name}.log"))
}

fn smoke_two_clients() -> Result<(), String> {
    client_build()?;

    let mut server = spawn(
        "spacetime",
        [
            "start",
            "--listen-addr",
            SPACETIME_LISTEN_ADDR,
            "--data-dir",
            ".spacetime-data",
            "--non-interactive",
        ],
        repo_dir(),
        &[],
    )?;

    let result = (|| {
        thread::sleep(Duration::from_millis(1200));
        db_publish()?;

        let mut grove = spawn_smoke_client("Grove", "hello from Grove", 1, 0, "flower")?;
        let mut moss = spawn_smoke_client("Moss", "hello from Moss", 0, 1, "button")?;
        wait_child("Godot smoke client Grove", &mut grove)?;
        wait_child("Godot smoke client Moss", &mut moss)?;

        let players = sql("SELECT * FROM player")?;
        require_contains(&players, "Grove", "player query")?;
        require_contains(&players, "Moss", "player query")?;

        let positions = sql("SELECT * FROM player_position")?;
        require_contains(&positions, "-368", "position query")?;

        let plots = sql("SELECT * FROM player_plot")?;
        require_contains(&plots, "640", "player plot query")?;

        let chat = sql("SELECT * FROM chat_message")?;
        require_contains(&chat, "hello from Grove", "chat query")?;
        require_contains(&chat, "hello from Moss", "chat query")?;
        require_contains(&chat, "inspects", "chat query")?;

        let objects = sql("SELECT * FROM world_tile")?;
        require_contains(&objects, "flower", "world tile query")?;
        require_contains(&objects, "button", "world tile query")?;

        println!(
            "smoke two-clients: replicated players, plots, positions, chat, and world objects"
        );
        Ok(())
    })();

    stop_child(&mut server);
    let _ = std::fs::remove_dir_all(repo_dir().join(".spacetime-data"));
    result
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

fn output<I, S>(program: &str, args: I, cwd: PathBuf) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run `{program}`: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "`{program}` exited with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn spawn<I, S>(program: &str, args: I, cwd: PathBuf, envs: &[(&str, &str)]) -> Result<Child, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().copied())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn `{program}`: {error}"))
}

fn wait_child(label: &str, child: &mut Child) -> Result<(), String> {
    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for {label}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} exited with {status}"))
    }
}

fn stop_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn spawn_smoke_client(
    name: &str,
    message: &str,
    dx: i32,
    dy: i32,
    object_kind: &str,
) -> Result<Child, String> {
    let dx = dx.to_string();
    let dy = dy.to_string();
    spawn(
        "godot",
        ["--headless", "--path", "godot", "--quit-after", "90"],
        repo_dir(),
        &[
            ("TINYGROVE_SMOKE", "1"),
            ("TINYGROVE_SMOKE_NAME", name),
            ("TINYGROVE_SMOKE_MESSAGE", message),
            ("TINYGROVE_SMOKE_DX", &dx),
            ("TINYGROVE_SMOKE_DY", &dy),
            ("TINYGROVE_SMOKE_OBJECT", object_kind),
            ("TINYGROVE_AGENT_PROFILE", name),
        ],
    )
}

fn sql(query: &str) -> Result<String, String> {
    output(
        "spacetime",
        ["sql", DB_NAME, query, "--server", "local", "--yes"],
        repo_dir(),
    )
}

fn require_contains(haystack: &str, needle: &str, label: &str) -> Result<(), String> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(format!(
            "{label} did not contain `{needle}`\noutput:\n{haystack}"
        ))
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
