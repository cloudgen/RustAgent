//! Domain verbs stay off the text screen.
//! requirement-domain-grok-cli, requirement-grok-auth-backup, requirement-sudoer-json-file.

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use rustagent::cli::{self, MemConsole};
use rustagent::domain::{Domain, RunReply};
use rustagent::host::Host;
use rustagent::identity::CONSOLE_NAME;

fn temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

fn write_exec(path: &std::path::Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut mode = fs::metadata(path).unwrap().permissions();
    mode.set_mode(0o755);
    fs::set_permissions(path, mode).unwrap();
}

#[test]
fn typed_domain_verbs_do_not_open_the_menu() {
    assert!(!cli::opens_text_screen(&["backup".into()], true));
    assert!(!cli::opens_text_screen(&["setup".into()], true));
    assert!(!cli::opens_text_screen(&["--json".into()], true));
    assert!(!cli::opens_text_screen(&["hello".into()], true));
    assert!(cli::opens_text_screen(&[], true));
    assert!(cli::opens_text_screen(&["about".into()], true));
    assert!(cli::opens_text_screen(&["menu".into()], true));
}

#[test]
fn unknown_join_and_restore() {
    for verb in ["join", "list-videos", "restore"] {
        let mut console = MemConsole::default();
        let code = cli::run(&[verb.into()], &mut console, Some("/tmp"), Some("/tmp"));
        assert_eq!(code, 1, "{verb}");
        assert!(console.err.contains("Unknown verb"), "{verb}");
    }
}

#[test]
fn sudoer_json_shape() {
    let home = temp_dir("agsudo-");
    let domain = Domain::new(map(&[("USER", "ada"), ("HOME", "/tmp")]), None, None, None, Some(1000));
    let mut domain = domain;
    let code = domain.generate_sudoer_request(Some(home.to_str().unwrap()), "add");
    assert_eq!(code, 0);
    let files: Vec<_> = fs::read_dir(&home).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(files.len(), 1);
    let name = files[0].to_string_lossy().into_owned();
    assert!(name.starts_with("sudoer-"));
    assert!(name.contains(&format!("-{CONSOLE_NAME}-ada-add-")));
    let doc: serde_json::Value = serde_json::from_str(&fs::read_to_string(home.join(&name)).unwrap()).unwrap();
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["service"], CONSOLE_NAME);
    assert_eq!(doc["commands"][0]["args"], serde_json::json!(["backup"]));
    assert_eq!(doc["commands"][0]["path"], format!("/usr/local/bin/{CONSOLE_NAME}"));
    assert_eq!(doc["commands"][0]["tags"], serde_json::json!(["NOPASSWD"]));
    let text = domain.sudoer_text(Some("ada"));
    assert!(text.contains("# Purpose: Allow ada to run rust-agent backup as root."));
    assert!(text.contains("ada ALL=(root) NOPASSWD: /usr/local/bin/rust-agent backup"));
}

#[test]
fn sync_auth_skips_when_logged_in() {
    let mut domain = Domain::new(map(&[("HOME", "/tmp")]), Some("valid".into()), None, None, Some(1000));
    assert_eq!(domain.sync_auth(), 0);
    assert!(domain.output.out.contains("No sync-auth for logged-in environment."));
    domain.session_override = Some("timeout".into());
    assert_eq!(domain.sync_auth(), 0);
}

#[test]
fn setup_platform_and_source_has_no_shell_installer() {
    let domain = Domain::new(map(&[("RUSTAGENT_PLATFORM", "linux-x86_64")]), None, None, None, Some(1000));
    assert_eq!(domain.detect_platform(), "linux-x86_64");
    let mac = Domain::new(map(&[("RUSTAGENT_PLATFORM", "macos-aarch64")]), None, None, None, Some(1000));
    assert_eq!(mac.detect_platform(), "macos-aarch64");
    let source = fs::read_to_string("src/domain.rs").unwrap();
    assert!(!source.contains("install.sh"));
    let wrapper = fs::read_to_string("src/wrapper_template.sh").unwrap();
    assert!(!wrapper.contains("install.sh"));
    assert!(wrapper.contains("proot-exit-reaper"));
}

#[test]
fn deposit_copies_auth_files() {
    let home = temp_dir("agauth-");
    let deposit = temp_dir("agdep-");
    let grok = home.join(".grok");
    fs::create_dir_all(&grok).unwrap();
    fs::write(grok.join("auth.json"), "{\"key\":\"x\"}\n").unwrap();
    let mut domain = Domain::new(
        map(&[("RUSTAGENT_BACKUP_NO_SUDO", "1"), ("HOME", home.to_str().unwrap())]),
        None,
        Some(deposit.to_string_lossy().into_owned()),
        Some(grok.to_string_lossy().into_owned()),
        Some(1000),
    );
    assert_eq!(domain.deposit_files(), 0);
    let stored = deposit.join("auth.json");
    assert!(stored.is_file());
    assert_eq!(fs::metadata(&stored).unwrap().permissions().mode() & 0o777, 0o644);
}

#[test]
fn remote_spec_four_forms() {
    let domain = Domain::new(map(&[("HOME", "/tmp"), ("USER", "ada")]), None, None, None, Some(1000));
    assert_eq!(domain.parse_remote_spec("ada@192.0.2.10"), Some(("ada".into(), "192.0.2.10".into())));
    assert_eq!(domain.parse_remote_spec("192.0.2.10"), Some(("".into(), "192.0.2.10".into())));
    assert_eq!(domain.parse_remote_spec("backup.example"), Some(("".into(), "backup.example".into())));
    assert_eq!(domain.parse_remote_spec("ada@backup.example"), Some(("ada".into(), "backup.example".into())));
    assert_eq!(domain.parse_remote_spec("ada@"), None);
    assert_eq!(domain.parse_remote_spec("-host"), None);
    assert_eq!(domain.parse_remote_spec("a b@host"), None);
}

#[test]
fn sync_from_remote_uses_scp_and_saves_preferred() {
    let home = temp_dir("agrem-");
    let bin_dir = temp_dir("agscp-");
    let store = home.join("remote-store");
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("auth.json"), "{\"key\":\"remote\"}\n").unwrap();
    let scp = bin_dir.join("scp");
    write_exec(
        &scp,
        "#!/bin/sh\nset -eu\ndest=\nsrc=\nskip=0\nfor arg in \"$@\"; do\n  if [ \"$skip\" -eq 1 ]; then skip=0; continue; fi\n  case \"$arg\" in\n    -o) skip=1 ;;\n    *) if [ -z \"$src\" ]; then src=\"$arg\"; else dest=\"$arg\"; fi ;;\n  esac\ndone\ncase \"$src\" in\n  *:*) cp \"${src#*:}\" \"$dest\" ;;\n  *) cp \"$src\" \"$dest\" ;;\nesac\n",
    );
    let mut domain = Domain::new(
        map(&[
            ("HOME", home.to_str().unwrap()),
            ("USER", "ada"),
            ("RUSTAGENT_SCP", scp.to_str().unwrap()),
            ("RUSTAGENT_REMOTE_ROOT", store.to_str().unwrap()),
            ("RUSTAGENT_CACHE_SKIP", "preferred"),
        ]),
        Some("missing".into()),
        None,
        Some(home.join(".grok").to_string_lossy().into_owned()),
        Some(1000),
    );
    let code = domain.sync_from_remote("ada@192.0.2.10");
    assert_eq!(code, 0, "{}", domain.output.out);
    let copied = home.join(".grok").join("auth.json");
    assert!(copied.is_file());
    assert_eq!(fs::metadata(&copied).unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!(domain.preferred_remote_load(), "ada@192.0.2.10");
    assert!(domain.output.out.contains("sync-auth-from-remote complete"));
}

#[test]
fn sync_to_remote_runs_when_logged_in() {
    let home = temp_dir("agto-");
    let remote_home = home.join("remote-login");
    fs::create_dir_all(&remote_home).unwrap();
    let grok = home.join(".grok");
    fs::create_dir_all(&grok).unwrap();
    fs::write(grok.join("auth.json"), "{\"key\":\"local\"}\n").unwrap();
    let bin_dir = temp_dir("agssh-");
    let ssh = bin_dir.join("ssh");
    let scp = bin_dir.join("scp");
    write_exec(
        &ssh,
        "#!/bin/sh\nset -eu\ntarget=\nskip=0\ncmd=\nfor arg in \"$@\"; do\n  if [ \"$skip\" -eq 1 ]; then skip=0; continue; fi\n  case \"$arg\" in\n    -o) skip=1 ;;\n    *) if [ -z \"$target\" ]; then target=\"$arg\"; else cmd=\"$arg\"; fi ;;\n  esac\ndone\ncd \"$REMOTE_HOME\"\neval \"$cmd\"\n",
    );
    write_exec(
        &scp,
        "#!/bin/sh\nset -eu\nsrc=\ndest=\nskip=0\nfor arg in \"$@\"; do\n  if [ \"$skip\" -eq 1 ]; then skip=0; continue; fi\n  case \"$arg\" in\n    -o) skip=1 ;;\n    *) if [ -z \"$src\" ]; then src=\"$arg\"; else dest=\"$arg\"; fi ;;\n  esac\ndone\ncase \"$dest\" in\n  *:*) cp \"$src\" \"$REMOTE_HOME/${dest#*:}\" ;;\n  *) cp \"$src\" \"$dest\" ;;\nesac\n",
    );
    let mut env = map(&[
        ("HOME", home.to_str().unwrap()),
        ("USER", "ada"),
        ("RUSTAGENT_SSH", ssh.to_str().unwrap()),
        ("RUSTAGENT_SCP", scp.to_str().unwrap()),
        ("RUSTAGENT_CACHE_SKIP", "preferred"),
    ]);
    env.insert("REMOTE_HOME".into(), remote_home.to_string_lossy().into_owned());
    let mut domain = Domain::new(
        env,
        Some("valid".into()),
        None,
        Some(grok.to_string_lossy().into_owned()),
        Some(1000),
    );
    let code = domain.sync_to_remote("ada@backup.example");
    assert_eq!(code, 0, "{}", domain.output.out);
    let stored = remote_home.join(".grok").join("auth.json");
    assert!(stored.is_file());
    assert_eq!(fs::metadata(&stored).unwrap().permissions().mode() & 0o777, 0o600);
    assert!(domain.output.out.contains("sync-auth-to-remote complete"));
}

#[test]
fn from_remote_skips_when_logged_in() {
    let mut domain = Domain::new(
        map(&[("HOME", "/tmp")]),
        Some("valid".into()),
        None,
        None,
        Some(1000),
    );
    assert_eq!(domain.sync_from_remote("ada@192.0.2.10"), 0);
    assert!(domain.output.out.contains("No sync-auth for logged-in environment."));
}

#[test]
fn add_crontab_installs_both_lines_once() {
    let home = temp_dir("agcron-");
    let cron_file = home.join("crontab.txt");
    fs::write(&cron_file, "").unwrap();
    let cron = home.join("crontab");
    write_exec(
        &cron,
        "#!/bin/sh\nset -eu\nif [ \"$1\" = \"-l\" ]; then\n  cat \"$CRON_FILE\"\n  exit 0\nfi\ncp \"$1\" \"$CRON_FILE\"\n",
    );
    let mut env = map(&[
        ("HOME", home.to_str().unwrap()),
        ("USER", "ada"),
        ("RUSTAGENT_CRONTAB", cron.to_str().unwrap()),
        ("RUSTAGENT_SUDO_GRANT", "1"),
        ("RUSTAGENT_ASSUME_BIN", "1"),
        ("RUSTAGENT_GLOBAL_BIN", "/usr/local/bin"),
        ("RUSTAGENT_CACHE_SKIP", "preferred"),
    ]);
    env.insert("CRON_FILE".into(), cron_file.to_string_lossy().into_owned());
    let mut domain = Domain::new(env, None, None, None, Some(1000));
    assert_eq!(domain.add_crontab(), 0, "{}", domain.output.err);
    let text = fs::read_to_string(&cron_file).unwrap();
    assert!(text.contains("sudo /usr/local/bin/rust-agent backup"), "{text}");
    assert!(text.contains("/usr/local/bin/rust-agent sync-auth"), "{text}");
    assert_eq!(domain.add_crontab(), 0);
    assert!(domain.output.out.contains("already present"));
    assert_eq!(fs::read_to_string(&cron_file).unwrap().matches("rust-agent backup").count(), 1);
}

#[test]
fn run_inserts_no_auto_update_and_update_needs_grok() {
    let peer = temp_dir("agpeer-");
    let binary = peer.join("grok");
    write_exec(&binary, "#!/bin/sh\nexit 0\n");
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let seen_run = std::sync::Arc::clone(&seen);
    let mut domain = Domain::new(
        map(&[("HOME", "/tmp"), ("RUSTAGENT_GROK_BIN", binary.to_str().unwrap())]),
        None,
        None,
        None,
        Some(1000),
    );
    domain.runner = Some(Box::new(move |argv| {
        *seen_run.lock().unwrap() = argv.to_vec();
        RunReply::Code(0)
    }));
    assert_eq!(domain.run(&["-p".into(), "hello".into()]), 0);
    let argv = seen.lock().unwrap().clone();
    assert_eq!(argv[0], binary.to_string_lossy());
    assert_eq!(argv[1], "--no-auto-update");
    assert!(argv.iter().any(|part| part == "-p"));
    let mut missing = Domain::new(
        map(&[("HOME", "/tmp"), ("RUSTAGENT_GROK_BIN", peer.join("missing").to_str().unwrap())]),
        None,
        None,
        None,
        Some(1000),
    );
    assert_eq!(missing.update_grok(), 1);
    assert!(missing.output.err.contains("not installed"));
}

#[test]
fn prefix_counts_as_termux_until_override() {
    let termux = Host::from_map(map(&[("PREFIX", "/data/data/com.termux/files/usr")]));
    assert!(termux.is_termux());
    assert!(termux.normal_user_only());
    let forced = Host::from_map(map(&[
        ("PREFIX", "/opt/local"),
        ("RUSTAGENT_HOST_OVERRIDE", "multiuser"),
    ]));
    assert!(!forced.is_termux());
    assert!(!forced.normal_user_only());
}

#[test]
fn android_wrapper_keeps_reaper_markers() {
    let folder = temp_dir("agwrap-");
    let domain = Domain::new(
        map(&[("HOME", folder.to_str().unwrap()), ("USER", "ada")]),
        None,
        None,
        None,
        Some(1000),
    );
    let path = folder.join("grok");
    assert!(domain.write_exec_wrapper(path.to_str().unwrap(), folder.join("vendor").to_str().unwrap(), ""));
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("proot-exit-reaper"));
    assert!(text.contains("--kill-on-exit"));
    assert!(text.contains("start-services.sh"));
    assert!(text.contains("unset LD_PRELOAD"));
    assert!(text.contains("TERMUX_EXEC_OPTOUT"));
    assert!(!text.contains("install.sh"));
}

#[test]
fn proot_reads_nested_cookies_and_skips_the_live_probe() {
    let home = temp_dir("agcookie-");
    let bindir = home.join("bin");
    fs::create_dir_all(&bindir).unwrap();
    write_exec(&bindir.join("proot"), "#!/bin/sh\nexit 0\n");
    let grok_home = home.join(".grok");
    fs::create_dir_all(&grok_home).unwrap();
    fs::write(
        grok_home.join("auth.json"),
        "{\"one\":{\"refresh_token\":\"cookie-value\"}}\n",
    ).unwrap();
    let called = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let called_run = std::sync::Arc::clone(&called);
    let mut domain = Domain::new(
        map(&[
            ("HOME", home.to_str().unwrap()),
            ("PATH", bindir.to_str().unwrap()),
            ("USER", "ada"),
            ("RUSTAGENT_HOST_OVERRIDE", ""),
        ]),
        None,
        None,
        Some(grok_home.to_string_lossy().into_owned()),
        Some(1000),
    );
    domain.runner = Some(Box::new(move |_| {
        called_run.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        RunReply::Code(1)
    }));
    assert_eq!(domain.session_word(), "valid");
    assert_eq!(called.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(domain.check_session(), 0);
    assert!(domain.output.out.contains("local auth cookies"));
}

#[test]
fn submit_queues_backup_and_refuses_a_wider_grant() {
    let home = temp_dir("agsub-");
    let inbound = home.join("inbound");
    fs::create_dir_all(&inbound).unwrap();
    let cli = home.join("sudoer-cli");
    write_exec(
        &cli,
        &format!(
            "#!/bin/sh\nfile=\nprev=\nfor arg in \"$@\"; do\n  if [ \"$prev\" = \"--file\" ]; then file=$arg; fi\n  prev=$arg\ndone\ncp \"$file\" \"{}/req-1\"\nprintf '%s\\n' 'request_id=req-1'\n",
            inbound.display()
        ),
    );
    let mut domain = Domain::new(
        map(&[
            ("HOME", home.to_str().unwrap()),
            ("USER", "ada"),
            ("PATH", "/usr/bin:/bin"),
            ("RUSTAGENT_SUDOER_CLI", cli.to_str().unwrap()),
            ("RUSTAGENT_SUDOER_INBOUND", inbound.to_str().unwrap()),
            ("RUSTAGENT_CACHE_SKIP", "preferred"),
            ("RUSTAGENT_GLOBAL_BIN", "/usr/local/bin"),
        ]),
        None,
        None,
        None,
        Some(1000),
    );
    assert_eq!(domain.submit_sudoer_request(""), 0, "{}", domain.output.err);
    let queued = inbound.join("req-1");
    let body: serde_json::Value = serde_json::from_str(&fs::read_to_string(&queued).unwrap()).unwrap();
    assert_eq!(body["commands"][0]["args"], serde_json::json!(["backup"]));
    assert!(domain.output.out.contains("did not write /etc"));
    assert!(domain.output.out.contains("request_id=req-1"));
    let wider = home.join("wider.json");
    fs::write(
        &wider,
        "{\"commands\":[{\"path\":\"/usr/local/bin/rust-agent\",\"args\":[\"backup\",\"restore\"]}]}",
    )
    .unwrap();
    assert_eq!(domain.submit_sudoer_request(wider.to_str().unwrap()), 1);
    assert!(domain.output.err.contains("does not allow backup"));
    let mut missing = Domain::new(
        map(&[
            ("HOME", home.to_str().unwrap()),
            ("USER", "ada"),
            ("RUSTAGENT_SUDOER_CLI", home.join("missing").to_str().unwrap()),
        ]),
        None,
        None,
        None,
        Some(1000),
    );
    assert_eq!(missing.submit_sudoer_request(""), 1);
    assert!(missing.output.err.contains("sudoer-cli not found"));
}

#[test]
fn remove_draft_needs_force_and_refuses_etc() {
    let home = temp_dir("agrm-");
    let draft_dir = home.join(".config").join(CONSOLE_NAME);
    fs::create_dir_all(&draft_dir).unwrap();
    let draft = draft_dir.join("sudoers.fragment-ada");
    fs::write(&draft, "ada ALL=(root) NOPASSWD: /usr/local/bin/rust-agent backup\n").unwrap();
    let mut domain = Domain::new(
        map(&[("HOME", home.to_str().unwrap()), ("USER", "ada")]),
        None,
        None,
        None,
        Some(1000),
    );
    assert_eq!(domain.remove_project_sudoers("", false), 1);
    assert!(draft.is_file());
    assert!(domain.output.err.contains("--force"));
    assert_eq!(domain.remove_project_sudoers("", true), 0);
    assert!(!draft.exists());
    assert!(domain.output.out.contains("Removed project-sudoers-file draft"));
    assert_eq!(domain.remove_project_sudoers("/etc/sudoers.d/rust-agent-ada", true), 1);
    assert!(domain.output.err.contains("Refusing to remove paths under /etc"));
}

#[test]
fn install_script_writes_draft_and_checks_with_sh() {
    let home = temp_dir("agscript-");
    let script = home.join("admin-sudoers");
    let mut domain = Domain::new(
        map(&[
            ("HOME", home.to_str().unwrap()),
            ("USER", "ada"),
            ("RUSTAGENT_CACHE_SKIP", "preferred"),
            ("RUSTAGENT_GLOBAL_BIN", "/usr/local/bin"),
        ]),
        None,
        None,
        None,
        Some(1000),
    );
    assert_eq!(domain.print_sudoers_install_script(script.to_str().unwrap()), 0, "{}", domain.output.err);
    let draft = home.join(".config").join(CONSOLE_NAME).join("sudoers.fragment-ada");
    assert!(draft.is_file());
    assert!(script.is_file());
    let body = fs::read_to_string(&script).unwrap();
    assert!(body.contains("cmd_install"));
    assert!(body.contains("cmd_uninstall"));
    assert!(body.contains("0440"));
    assert!(body.contains("/etc/sudoers.d/rust-agent-ada"));
    assert!(!body.contains("install.sh"));
    let proc = Command::new("sh").arg("-n").arg(&script).status().unwrap();
    assert!(proc.success());
    assert!(domain.output.out.contains(&format!("sudo sh {} install", script.display())));
}

#[test]
fn cli_accepts_force_on_remove_project_sudoers() {
    let home = temp_dir("agcli-");
    let previous_home = std::env::var("HOME").ok();
    let previous_user = std::env::var("USER").ok();
    std::env::set_var("HOME", &home);
    std::env::set_var("USER", "ada");
    std::env::set_var("RUSTAGENT_CACHE_SKIP", "preferred");
    let mut console = MemConsole::default();
    let code = cli::run(
        &["remove-project-sudoers".into(), "--force".into()],
        &mut console,
        Some("/tmp"),
        Some("/tmp"),
    );
    if let Some(value) = previous_home {
        std::env::set_var("HOME", value);
    }
    if let Some(value) = previous_user {
        std::env::set_var("USER", value);
    }
    std::env::remove_var("RUSTAGENT_CACHE_SKIP");
    assert_eq!(code, 0, "{}{}", console.out, console.err);
    assert!(console.out.contains("nothing to remove") || console.out.contains("Removed"));
}
