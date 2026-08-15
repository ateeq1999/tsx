use super::args::*;
use clap::CommandFactory;

/// Parse a `--json` argument, printing a structured error and returning `None` on failure.
/// This replaces the old `serde_json::from_str(&json.unwrap()).unwrap()` pattern that panics.
fn parse_json_input<T: serde::de::DeserializeOwned>(
    json: Option<String>,
    json_input: Option<&str>,
    cmd_name: &str,
) -> Option<T> {
    use tsx::json::error::ErrorResponse;
    use tsx::json::response::ResponseEnvelope;

    let raw = match json.as_deref().or(json_input) {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => {
            let err =
                ErrorResponse::validation(&format!("'{}' requires --json <JSON>", cmd_name));
            ResponseEnvelope::error(cmd_name, err, 0).print();
            return None;
        }
    };
    match serde_json::from_str(&raw) {
        Ok(v) => Some(v),
        Err(e) => {
            let err = ErrorResponse::validation(&format!(
                "'{}' received invalid JSON — {}",
                cmd_name, e
            ));
            ResponseEnvelope::error(cmd_name, err, 0).print();
            None
        }
    }
}

/// Route a parsed `Cli` invocation to its command handler.
pub fn dispatch(cli: Cli, json_input: Option<String>) {
    if cli.dry_run {
        println!("Dry run mode - no files will be written");
    }

    match cli.command {
        Command::Init { name, stack } => {
            use tsx::commands::init;
            let result = init::init(name, stack);
            result.print();
        }
        Command::Dev {
            json_events,
            watch,
            ws_port,
        } => {
            use tsx::commands::dev;
            let result = dev::dev(json_events, watch, ws_port);
            result.print();
        }
        Command::Generate { generator } => {
            // All generate subcommands delegate to the universal `tsx run` dispatcher.
            use tsx::commands::run;
            let (cmd_id, json) = match generator {
                Generate::Feature { json }   => ("add:feature",   json),
                Generate::Schema { json }    => ("add:schema",    json),
                Generate::ServerFn { json }  => ("add:server-fn", json),
                Generate::Query { json }     => ("add:query",     json),
                Generate::Form { json }      => ("add:form",      json),
                Generate::Table { json }     => ("add:table",     json),
                Generate::Page { json }      => ("add:page",      json),
                Generate::Seed { json }      => ("add:seed",      json),
                Generate::Fw { id, fw, json } => {
                    let merged = json.or_else(|| json_input.clone());
                    run::run(id, fw, merged, cli.overwrite, cli.dry_run, cli.verbose).print();
                    return;
                }
            };
            let merged = json.or_else(|| json_input.clone());
            run::run(cmd_id.to_string(), None, merged, cli.overwrite, cli.dry_run, cli.verbose).print();
        }
        Command::Add { integration } => {
            use tsx::commands::run;
            match integration {
                Add::Auth { json } => {
                    let merged = json.or_else(|| json_input.clone());
                    run::run("add:auth".to_string(), None, merged, cli.overwrite, cli.dry_run, cli.verbose).print();
                }
                Add::AuthGuard { json } => {
                    let merged = json.or_else(|| json_input.clone());
                    run::run("add:auth-guard".to_string(), None, merged, cli.overwrite, cli.dry_run, cli.verbose).print();
                }
                Add::Migration => {
                    use tsx::commands::add_migration;
                    add_migration::add_migration().print();
                }
            }
        }
        Command::List { kind } => {
            use tsx::commands::list;
            let result = list::list(kind, cli.verbose);
            result.print();
        }

        Command::Create { from, starter } => {
            use tsx::commands::create;
            let result = create::create(from, starter, cli.dry_run, cli.verbose);
            result.print();
        }
        Command::Framework { action } => match action {
            FrameworkCmd::Init { name } => {
                use tsx::commands::framework;
                let result = framework::framework_init(name, cli.verbose);
                result.print();
            }
            FrameworkCmd::Validate { path } => {
                use tsx::commands::framework;
                let result = framework::framework_validate(path, cli.verbose);
                result.print();
            }
            FrameworkCmd::Preview { template, data } => {
                use tsx::commands::framework;
                let result = framework::framework_preview(template, data, cli.verbose);
                result.print();
            }
            FrameworkCmd::Add { source } => {
                use tsx::commands::framework;
                let result = framework::framework_add(source, cli.verbose);
                result.print();
            }
            FrameworkCmd::List => {
                use tsx::commands::framework;
                let result = framework::framework_list(cli.verbose);
                result.print();
            }
            FrameworkCmd::Publish { path, dry_run, registry, api_key } => {
                use tsx::commands::framework;
                let result = framework::framework_publish(path, dry_run, registry, api_key, cli.verbose);
                result.print();
            }
        },
        Command::Inspect => {
            use tsx::commands::inspect;
            let result = inspect::inspect(cli.verbose);
            result.print();
        }
        Command::Batch { json, stream, plan } => {
            use tsx::commands::batch;
            use tsx::json::payload::BatchPayload;
            let ji = json_input.as_deref();
            if let Some(payload) = parse_json_input::<BatchPayload>(json, ji, "batch") {
                if plan || cli.dry_run {
                    batch::batch_plan(payload, cli.verbose).print();
                } else {
                    batch::batch(payload, cli.overwrite, false, cli.verbose, stream).print();
                }
            }
        }
        Command::Subscribe { port } => {
            use tsx::commands::subscribe;
            let result = subscribe::subscribe(port, cli.verbose);
            result.print();
        }
        Command::Describe { target, framework, section } => {
            use tsx::commands::query::describe;
            // Resolve: positional arg takes precedence over --framework flag
            let resolved = target.or(framework);
            let result = describe::describe(resolved, section, cli.verbose);
            result.print();
        }
        Command::Ask {
            question,
            framework,
            depth,
        } => {
            use tsx::commands::query::ask;
            // Auto-detect framework from package.json when not specified
            let resolved_framework = framework.or_else(|| {
                let root = std::env::current_dir().ok()?;
                tsx::framework::detect::detect_framework(&root)
            });
            let result = ask::ask(question, resolved_framework, depth, cli.verbose);
            result.print();
        }
        Command::Where { thing, framework } => {
            use tsx::commands::query::where_cmd;
            let result = where_cmd::where_cmd(thing, framework, cli.verbose);
            result.print();
        }
        Command::How {
            integration,
            framework,
        } => {
            use tsx::commands::query::how;
            let result = how::how(integration, framework, cli.verbose);
            result.print();
        }
        Command::Explain { topic } => {
            use tsx::commands::query::explain;
            let result = explain::explain(topic, cli.verbose);
            result.print();
        }
        Command::Upgrade { target } => match target {
            Upgrade::Atoms { check } => {
                use tsx::commands::upgrade;
                let result = upgrade::upgrade(check, cli.verbose);
                result.print();
            }
            Upgrade::Cli { check } => {
                use tsx::commands::self_update;
                let result = self_update::self_update(check);
                result.print();
            }
        },
        Command::Publish { action } => match action {
            Publish::Registry { registry, output } => {
                use tsx::commands::publish;
                let result = publish::publish(registry, output, cli.verbose);
                result.print();
            }
            Publish::List => {
                use tsx::commands::publish;
                let result = publish::publish_list(cli.verbose);
                result.print();
            }
        },
        Command::Registry { action } => match action {
            RegistryCmd::Search { query } => {
                use tsx::commands::registry;
                let result = registry::registry_search(query, cli.verbose);
                result.print();
            }
            RegistryCmd::Install { package } => {
                use tsx::commands::registry;
                let result = registry::registry_install(package, cli.verbose);
                result.print();
            }
            RegistryCmd::List => {
                use tsx::commands::registry;
                let result = registry::registry_list(cli.verbose);
                result.print();
            }
            RegistryCmd::Website { output } => {
                use tsx::commands::registry;
                let result = registry::registry_website(output, cli.verbose);
                result.print();
            }
            RegistryCmd::Update => {
                use tsx::commands::registry;
                let result = registry::registry_update(cli.verbose);
                result.print();
            }
            RegistryCmd::Info { package } => {
                use tsx::commands::registry;
                let result = registry::registry_info(package, cli.verbose);
                result.print();
            }
        },
        Command::Login { token, registry } => {
            use tsx::commands::auth;
            auth::login(token, registry).print();
        }
        Command::Logout => {
            use tsx::commands::auth;
            auth::logout().print();
        }
        Command::Whoami => {
            use tsx::commands::auth;
            auth::whoami().print();
        }
        Command::Pkg { action } => match action {
            PkgCmd::Install { name, version, target } => {
                use tsx::commands::pkg;
                pkg::pkg_install(name, version, target).print();
            }
            PkgCmd::Info { name } => {
                use tsx::commands::pkg;
                pkg::pkg_info(name).print();
            }
            PkgCmd::Upgrade { name, target } => {
                use tsx::commands::pkg;
                pkg::pkg_upgrade(name, target).print();
            }
            PkgCmd::Publish { path, name, version, dry_run } => {
                use tsx::commands::pkg;
                pkg::pkg_publish(path, name, version, dry_run).print();
            }
        },
        Command::Plugin { action } => match action {
            Plugin::List => {
                use tsx::commands::plugin;
                let result = plugin::plugin_list(cli.verbose);
                result.print();
            }
            Plugin::Install { source } => {
                use tsx::commands::plugin;
                let result = plugin::plugin_install(source, cli.verbose);
                result.print();
            }
            Plugin::Remove { package } => {
                use tsx::commands::plugin;
                let result = plugin::plugin_remove(package, cli.verbose);
                result.print();
            }
        },
        Command::Stack { action } => match action {
            StackCmd::Init { lang, packages } => {
                use tsx::commands::stack;
                stack::stack_init(lang, packages, cli.dry_run, cli.verbose).print();
            }
            StackCmd::Show => {
                use tsx::commands::stack;
                stack::stack_show(cli.verbose).print();
            }
            StackCmd::Add { package } => {
                use tsx::commands::stack;
                stack::stack_add(package, cli.verbose).print();
            }
            StackCmd::Remove { package } => {
                use tsx::commands::stack;
                stack::stack_remove(package, cli.verbose).print();
            }
            StackCmd::Detect { install } => {
                use tsx::commands::stack;
                stack::stack_detect(install, cli.verbose).print();
            }
        },
        Command::Plan { json } => {
            use tsx::commands::plan;
            use tsx::commands::plan::PlanGoal;
            let ji = json_input.as_deref();
            if let Some(goals) = parse_json_input::<Vec<PlanGoal>>(json, ji, "plan") {
                plan::plan(goals, cli.verbose).print();
            }
        }
        Command::Context => {
            use tsx::commands::context;
            context::context(cli.verbose).print();
        }
        Command::Completions { shell } => {
            use clap_complete::generate;
            use std::io::Write;
            use tsx::commands::completions;

            match completions::resolve_shell(&shell) {
                Err(_) => completions::unknown_shell_error(&shell).print(),
                Ok(sh) => {
                    let mut cmd = Cli::command();
                    let mut buf = Vec::<u8>::new();
                    generate(sh, &mut cmd, "tsx", &mut buf);
                    std::io::stdout().write_all(&buf).ok();
                }
            }
        }
        Command::Doctor => {
            use tsx::commands::doctor;
            doctor::doctor().print();
        }
        Command::LintTemplate { path } => {
            use tsx::commands::lint_template;
            lint_template::lint_template(path, cli.verbose).print();
        }
        Command::Snapshot { action } => match action {
            SnapshotCmd::Update { generator } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_update(generator, cli.verbose).print();
            }
            SnapshotCmd::Diff { generator } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_diff(generator, cli.verbose).print();
            }
            SnapshotCmd::Accept { generator } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_accept(generator, cli.verbose).print();
            }
            SnapshotCmd::List => {
                use tsx::commands::snapshot;
                snapshot::snapshot_list(cli.verbose).print();
            }
            SnapshotCmd::Add { generator, fixture, input } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_add(generator, fixture, input, cli.verbose).print();
            }
        },
        Command::Pattern { action } => match action {
            PatternCmd::New { id, name, description, framework } => {
                use tsx::commands::pattern;
                pattern::pattern_new(id, name, description, framework, cli.verbose).print();
            }
            PatternCmd::Run { id, command, args } => {
                use tsx::commands::pattern;
                pattern::pattern_run(id, command, args, cli.dry_run, cli.overwrite, cli.diff, cli.verbose).print();
            }
            PatternCmd::Install { source, id } => {
                use tsx::commands::pattern;
                pattern::pattern_install(source, id, cli.verbose).print();
            }
            PatternCmd::Lint { id } => {
                use tsx::commands::pattern;
                pattern::pattern_lint(id, cli.verbose).print();
            }
            PatternCmd::Add { name, description, template, args } => {
                use tsx::commands::pattern;
                pattern::pattern_add(name, description, template, args, cli.verbose).print();
            }
            PatternCmd::Record { name, stop } => {
                use tsx::commands::pattern;
                if stop {
                    pattern::pattern_record_stop(cli.verbose).print();
                } else {
                    match name {
                        Some(n) => pattern::pattern_record_start(n, cli.verbose).print(),
                        None => {
                            use tsx::json::error::{ErrorCode, ErrorResponse};
                            use tsx::json::response::ResponseEnvelope;
                            ResponseEnvelope::error(
                                "pattern record",
                                ErrorResponse::new(ErrorCode::ValidationError, "--name is required when starting a recording"),
                                0,
                            ).print();
                        }
                    }
                }
            }
            PatternCmd::List { builtin } => {
                use tsx::commands::pattern;
                if builtin {
                    pattern::pattern_list_builtin(cli.verbose).print();
                } else {
                    pattern::pattern_list(cli.verbose).print();
                }
            }
            PatternCmd::Show { id } => {
                use tsx::commands::pattern;
                pattern::pattern_show(id, cli.verbose).print();
            }
            PatternCmd::Remove { id } => {
                use tsx::commands::pattern;
                pattern::pattern_remove(id, cli.verbose).print();
            }
            PatternCmd::Publish { id, registry } => {
                use tsx::commands::pattern;
                pattern::pattern_publish(id, registry, cli.verbose).print();
            }
            PatternCmd::Search { query, framework, registry } => {
                use tsx::commands::pattern;
                pattern::pattern_search(query, registry, framework, cli.verbose).print();
            }
            PatternCmd::Share { name, version } => {
                use tsx::commands::pattern;
                pattern::pattern_share(name, version, cli.verbose).print();
            }
            PatternCmd::Eject { id } => {
                use tsx::commands::pattern;
                pattern::pattern_eject(id, cli.verbose).print();
            }
            PatternCmd::Update { id } => {
                use tsx::commands::pattern;
                pattern::pattern_update(id, cli.verbose).print();
            }
        },
        Command::Codegen { target } => match target {
            CodegenCmd::RustToTs { input, out, watch } => {
                use tsx::commands::codegen;
                codegen::codegen_rust_to_ts(input, out, watch, cli.verbose).print();
            }
            CodegenCmd::OpenapiToZod { spec, out } => {
                use tsx::commands::codegen;
                codegen::codegen_openapi_to_zod(spec, out, cli.verbose).print();
            }
            CodegenCmd::DrizzleToZod => {
                use tsx::commands::codegen;
                codegen::codegen_drizzle_to_zod(cli.verbose).print();
            }
        },
        Command::Run { id, fw, json, list } => {
            use tsx::commands::run;
            if list || id.is_none() {
                run::run_list(fw, cli.verbose).print();
            } else {
                let merged_json = json.or_else(|| json_input.clone());
                run::run(
                    id.unwrap(),
                    fw,
                    merged_json,
                    cli.overwrite,
                    cli.dry_run,
                    cli.verbose,
                )
                .print();
            }
        }
        Command::Docs { paths, search, json } => {
            use tsx::commands::docs;
            docs::docs(paths, search, json);
        }
        Command::Fmt { paths, check, indent, quotes } => {
            use tsx::commands::fmt;
            fmt::fmt(paths, check, indent, quotes);
        }
        Command::Watch { paths, ext, debounce, run: run_cmd, json } => {
            use tsx::commands::watch;
            watch::watch(paths, ext, debounce, run_cmd, json);
        }
        Command::Tui { view } => {
            use tsx::commands::tui;
            tui::tui(view);
        }
        Command::Config { action } => {
            use tsx::commands::config;
            match action {
                ConfigCmd::Get { key } => config::config_get(key).print(),
                ConfigCmd::Set { key, value } => config::config_set(key, value).print(),
                ConfigCmd::List => config::config_list().print(),
                ConfigCmd::Reset { key } => config::config_reset(key).print(),
            }
        }
        Command::Env { action } => {
            use tsx::commands::env;
            match action {
                EnvCmd::Check { schema, env: env_path } => env::env_check(schema, env_path).print(),
                EnvCmd::Diff { example, env: env_path } => env::env_diff(example, env_path).print(),
            }
        }
        Command::Migrate { generate_only, apply_only } => {
            use tsx::commands::migrate;
            migrate::migrate(generate_only, apply_only, cli.dry_run, cli.verbose).print();
        }
        Command::Build { json_events } => {
            use tsx::commands::build;
            build::build(json_events, cli.verbose).print();
        }
        Command::Test { filter, watch, json } => {
            use tsx::commands::test_run;
            test_run::test_run(filter, watch, json, cli.verbose).print();
        }
        Command::Audit { severity, fix } => {
            use tsx::commands::audit;
            audit::audit(severity, fix, cli.verbose).print();
        }
        Command::Repl { goal, execute } => {
            use tsx::commands::repl;
            repl::repl(goal, execute, cli.verbose).print();
        }
        Command::Atoms { action } => {
            use tsx::commands::atoms;
            match action {
                AtomsCmd::List { category } => atoms::atoms_list(category, cli.verbose).print(),
                AtomsCmd::Preview { id } => atoms::atoms_preview(id, cli.verbose).print(),
            }
        }
        Command::Analyze { fix, report } => {
            use tsx::commands::analyze;
            analyze::analyze(fix, report, cli.verbose).print();
        }
        Command::Replay { action } => {
            use tsx::commands::replay;
            match action {
                ReplayCmd::Record { out, stop } => {
                    if stop {
                        replay::replay_record_stop(cli.verbose).print();
                    } else {
                        replay::replay_record_start(out, cli.verbose).print();
                    }
                }
                ReplayCmd::Run { file, dry_run } => {
                    replay::replay_run(file, dry_run, cli.verbose).print();
                }
                ReplayCmd::List => replay::replay_list(cli.verbose).print(),
            }
        }
        Command::Lsp => {
            if let Err(e) = tsx_lsp::run_lsp_server() {
                eprintln!("tsx-lsp: fatal error: {}", e);
                std::process::exit(1);
            }
        }
        Command::Package { action } => {
            use tsx::commands::package;
            match action {
                PackageCmd::New { id, out } => package::package_new(id, out).print(),
                PackageCmd::Validate { dir } => package::package_validate(dir).print(),
                PackageCmd::Pack { dir, out } => package::package_pack(dir, out).print(),
                PackageCmd::Publish { dir, registry, token } => {
                    package::package_publish(dir, registry, token).print()
                }
                PackageCmd::Install { id, registry } => {
                    package::package_install(id, registry).print()
                }
            }
        }
        Command::Mcp => {
            use tsx::commands::mcp;
            mcp::run_mcp_server();
        }
        Command::Template { action } => {
            use tsx::commands::template;
            match action {
                TemplateCmd::List { source } => {
                    template::template_list(source, cli.verbose).print();
                }
                TemplateCmd::Info { name } => {
                    template::template_info(name, cli.verbose).print();
                }
                TemplateCmd::Init { name, dest } => {
                    template::template_init(name, dest, cli.verbose).print();
                }
                TemplateCmd::Install { source } => {
                    template::template_install(source, cli.verbose).print();
                }
                TemplateCmd::Uninstall { name } => {
                    template::template_uninstall(name, cli.verbose).print();
                }
                TemplateCmd::Schema { name, command } => {
                    template::template_schema(name, command, cli.verbose).print();
                }
                TemplateCmd::Lint { path } => {
                    template::template_lint(path, cli.verbose).print();
                }
                TemplateCmd::Config { action } => match action {
                    TemplateConfigCmd::Show => {
                        template::template_config_show(cli.verbose).print();
                    }
                    TemplateConfigCmd::Set { key, value } => {
                        template::template_config_set(key, value, cli.verbose).print();
                    }
                    TemplateConfigCmd::Init { overwrite } => {
                        template::template_config_init(overwrite, cli.verbose).print();
                    }
                },
                TemplateCmd::Login { token, registry } => {
                    template::template_login(token, registry, cli.verbose).print();
                }
                TemplateCmd::Logout => {
                    template::template_logout(cli.verbose).print();
                }
                TemplateCmd::Publish { name, version, path } => {
                    template::template_publish(name, version, path, cli.verbose).print();
                }
            }
        }
        Command::Path { directory, permanent, list } => {
            use tsx::commands::path;
            if list {
                path::path_list().print();
            } else {
                path::path_add(directory, permanent).print();
            }
        }
        Command::Adb { action } => {
            use tsx::commands::adb;
            match action {
                AdbCmd::Kill => adb::adb_kill().print(),
                AdbCmd::Start => adb::adb_start().print(),
                AdbCmd::Status => adb::adb_status().print(),
                AdbCmd::Reverse { port } => adb::adb_reverse(port).print(),
                AdbCmd::Exec { args } => adb::adb_exec(args).print(),
            }
        }
        Command::Flutter { action } => {
            use tsx::commands::flutter;
            match action {
                FlutterCmd::Run { device, mode, port } => {
                    flutter::flutter_run(mode, device, port).print();
                }
                FlutterCmd::Build { target, release } => {
                    flutter::flutter_build(target, release).print();
                }
                FlutterCmd::Clean => flutter::flutter_clean().print(),
                FlutterCmd::PubGet => flutter::flutter_pub_get().print(),
            }
        }
        Command::Port { action } => {
            use tsx::commands::port;
            match action {
                PortCmd::Find { port } => port::find_process_by_port(port).print(),
                PortCmd::Kill { port } => port::kill_all_port(port).print(),
            }
        }
    }
}
