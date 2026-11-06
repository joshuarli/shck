pub use crate::cli::check::CheckCommand;
pub use crate::cli::clean::CleanCommand;
pub use crate::cli::format::FormatCommand;
pub use crate::cli::install::InstallCommand;
pub use crate::cli::run::RunCommand;
pub use crate::cli::shell::ShellCommand;
pub use crate::cli::types::{
    CheckOutputFormatArg, FileSelectionArgs, FormatDialectArg, FormatIndentStyleArg,
    FrameworkRootPair, ManagedShellArg, PatternFrameworkNameTriple, PatternPathPair,
    PatternRuleSelectorPair, PatternShellPair, RuleSelectionArgs, TerminalColor,
    parse_cli_rule_selector,
};
pub use crate::cli::{Args, Command};

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use tempfile::tempdir;

    use shuck_config::{ConfigArguments, SingleConfigArgument};

    use super::*;

    #[test]
    fn global_config_override_is_available_after_subcommand() {
        let override_argument = SingleConfigArgument::parse(std::ffi::OsStr::new("format.indent-width = 2"))
            .unwrap();

        let args = Args::try_parse_from(["shuck", "check", "--config", "format.indent-width = 2"])
            .unwrap();

        assert_eq!(
            args.config,
            ConfigArguments::from_cli(vec![override_argument], false).unwrap()
        );
    }

    #[test]
    fn explicit_config_file_and_inline_override_both_parse_globally() {
        let tempdir = tempdir().unwrap();
        let config_path = tempdir.path().join("shuck.toml");
        fs::write(&config_path, "[format]\nfunction-next-line = true\n").unwrap();

        let override_argument = SingleConfigArgument::parse(std::ffi::OsStr::new("format.indent-width = 2"))
            .unwrap();

        let args = Args::try_parse_from([
            "shuck".into(),
            "--config".into(),
            config_path.as_os_str().to_os_string(),
            "check".into(),
            "--config".into(),
            "format.indent-width = 2".into(),
        ])
        .unwrap();

        assert_eq!(
            args.config,
            ConfigArguments::from_cli(
                vec![
                    SingleConfigArgument::FilePath(config_path),
                    override_argument,
                ],
                false,
            )
            .unwrap()
        );
    }

    #[test]
    fn global_color_can_be_parsed_before_subcommand() {
        let args = Args::try_parse_from(["shuck", "--color", "always", "check"]).unwrap();
        assert_eq!(args.color, Some(TerminalColor::Always));
    }

    fn parse_check<I, T>(args: I) -> CheckCommand
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        match Args::try_parse_from(args).unwrap().command {
            Command::Check(command) => *command,
            _ => unreachable!(),
        }
    }

    fn parse_run<I, T>(args: I) -> RunCommand
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        match Args::try_parse_from(args).unwrap().command {
            Command::Run(command) => command,
            _ => unreachable!(),
        }
    }

    fn parse_install<I, T>(args: I) -> InstallCommand
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        match Args::try_parse_from(args).unwrap().command {
            Command::Install(command) => command,
            _ => unreachable!(),
        }
    }

    fn parse_shell<I, T>(args: I) -> ShellCommand
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        match Args::try_parse_from(args).unwrap().command {
            Command::Shell(command) => command,
            _ => unreachable!(),
        }
    }

    #[test]
    fn parses_busybox_shell_variants() {
        let run_command = parse_run(["shuck", "run", "--shell", "busybox", "deploy.sh"]);
        assert_eq!(run_command.shell, Some(ManagedShellArg::Busybox));

        let install_command = parse_install(["shuck", "install", "busybox", "1.36.1"]);
        assert_eq!(install_command.shell, Some(ManagedShellArg::Busybox));

        let shell_command = parse_shell(["shuck", "shell", "--shell", "busybox"]);
        assert_eq!(shell_command.shell, Some(ManagedShellArg::Busybox));
    }

    #[test]
    fn parses_extended_managed_shell_names() {
        for name in ["gbash", "bashkit", "dash", "mksh"] {
            let run_command = parse_run(["shuck", "run", "--shell", name, "deploy.sh"]);
            assert_eq!(
                run_command.shell,
                Some(name.parse::<ManagedShellArg>().unwrap())
            );

            let install_command = parse_install(["shuck", "install", name, "latest"]);
            assert_eq!(
                install_command.shell,
                Some(name.parse::<ManagedShellArg>().unwrap())
            );

            let shell_command = parse_shell(["shuck", "shell", "--shell", name]);
            assert_eq!(
                shell_command.shell,
                Some(name.parse::<ManagedShellArg>().unwrap())
            );
        }
    }

    #[test]
    fn parses_rule_selection_flags() {
        let command = parse_check([
            "shuck",
            "check",
            "--select",
            "C001,google",
            "--ignore",
            "S001",
            "--extend-select",
            "ALL",
            "--fixable",
            "C001",
            "--unfixable",
            "google",
            "--extend-fixable",
            "ALL",
        ]);

        assert_eq!(
            command.rule_selection.select,
            Some(vec![
                parse_cli_rule_selector("C001").unwrap(),
                parse_cli_rule_selector("google").unwrap(),
            ])
        );
        assert_eq!(
            command.rule_selection.ignore,
            vec![parse_cli_rule_selector("S001").unwrap()]
        );
        assert_eq!(
            command.rule_selection.extend_select,
            vec![parse_cli_rule_selector("ALL").unwrap()]
        );
        assert_eq!(
            command.rule_selection.fixable,
            Some(vec![parse_cli_rule_selector("C001").unwrap()])
        );
        assert_eq!(
            command.rule_selection.unfixable,
            vec![parse_cli_rule_selector("google").unwrap()]
        );
        assert_eq!(
            command.rule_selection.extend_fixable,
            vec![parse_cli_rule_selector("ALL").unwrap()]
        );
    }

    #[test]
    fn parses_named_rule_selection_flags() {
        let command = parse_check([
            "shuck",
            "check",
            "--select",
            "google",
            "--extend-select",
            "google",
            "--fixable",
            "google",
        ]);

        assert_eq!(
            command.rule_selection.select,
            Some(vec![shuck_linter::RuleSelector::Named(shuck_linter::NamedGroup::Google)])
        );
        assert_eq!(
            command.rule_selection.extend_select,
            vec![shuck_linter::RuleSelector::Named(shuck_linter::NamedGroup::Google)]
        );
        assert_eq!(
            command.rule_selection.fixable,
            Some(vec![shuck_linter::RuleSelector::Named(shuck_linter::NamedGroup::Google)])
        );
    }

    #[test]
    fn parses_per_file_ignore_pairs() {
        let command = parse_check([
            "shuck",
            "check",
            "--per-file-ignores",
            "a.sh:C001,b.sh:google",
            "--extend-per-file-ignores",
            "c.sh:ALL",
        ]);

        assert_eq!(
            command.rule_selection.per_file_ignores,
            Some(vec![
                PatternRuleSelectorPair {
                    pattern: "a.sh".to_owned(),
                    selector: parse_cli_rule_selector("C001").unwrap(),
                },
                PatternRuleSelectorPair {
                    pattern: "b.sh".to_owned(),
                    selector: parse_cli_rule_selector("google").unwrap(),
                },
            ])
        );
        assert_eq!(
            command.rule_selection.extend_per_file_ignores,
            vec![PatternRuleSelectorPair {
                pattern: "c.sh".to_owned(),
                selector: parse_cli_rule_selector("ALL").unwrap(),
            }]
        );
    }

    #[test]
    fn parses_named_per_file_ignore_pairs() {
        let command = parse_check(["shuck", "check", "--per-file-ignores", "tests/*.sh:google"]);

        assert_eq!(
            command.rule_selection.per_file_ignores,
            Some(vec![PatternRuleSelectorPair {
                pattern: "tests/*.sh".to_owned(),
                selector: shuck_linter::RuleSelector::Named(shuck_linter::NamedGroup::Google),
            }])
        );
    }

    #[test]
    fn parses_per_file_ignore_pairs_with_colons_in_pattern() {
        let command = parse_check([
            "shuck",
            "check",
            "--per-file-ignores",
            "path:with:colons.sh:C001",
        ]);

        assert_eq!(
            command.rule_selection.per_file_ignores,
            Some(vec![PatternRuleSelectorPair {
                pattern: "path:with:colons.sh".to_owned(),
                selector: parse_cli_rule_selector("C001").unwrap(),
            }])
        );
    }

    #[test]
    fn parses_per_file_shell_pairs() {
        let command = parse_check([
            "shuck",
            "check",
            "--per-file-shell",
            "a.sh:bash,b.sh:mksh",
            "--extend-per-file-shell",
            "c.sh:mksh",
        ]);

        assert_eq!(
            command.rule_selection.per_file_shell,
            Some(vec![
                PatternShellPair {
                    pattern: "a.sh".to_owned(),
                    shell: shuck_linter::ShellDialect::Bash,
                },
                PatternShellPair {
                    pattern: "b.sh".to_owned(),
                    shell: shuck_linter::ShellDialect::Mksh,
                },
            ])
        );
        assert_eq!(
            command.rule_selection.extend_per_file_shell,
            vec![PatternShellPair {
                pattern: "c.sh".to_owned(),
                shell: shuck_linter::ShellDialect::Mksh,
            }]
        );
    }



    #[test]
    fn check_file_selection_collects_exclude_and_extend_exclude_patterns() {
        let command = parse_check([
            "shuck",
            "check",
            "--exclude",
            "foo.sh,bar.sh",
            "--extend-exclude",
            "baz.sh",
        ]);

        assert_eq!(
            command.file_selection.exclude,
            vec!["foo.sh".to_owned(), "bar.sh".to_owned()]
        );
        assert_eq!(
            command.file_selection.extend_exclude,
            vec!["baz.sh".to_owned()]
        );
    }
}
