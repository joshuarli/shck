use std::fmt::Write as _;

use crate::cli::command::{CommandSpec, OptionSpec, PositionalSpec};

pub fn render_command_help(
    bin_name: &str,
    spec: &CommandSpec,
    common_options: &[OptionSpec],
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", spec.description);
    let _ = writeln!(out);

    render_usage(&mut out, bin_name, spec);
    let _ = writeln!(out);

    if !spec.positionals.is_empty() {
        let _ = writeln!(out, "Arguments:");
        for pos in spec.positionals {
            render_positional(&mut out, pos);
            let _ = writeln!(out);
        }
    }

    render_option_groups(&mut out, spec.options, false);

    if !common_options.is_empty() {
        let _ = writeln!(out, "Global options:");
        for opt in common_options {
            if !opt.hidden {
                render_option(&mut out, opt);
                let _ = writeln!(out);
            }
        }
    }

    if !spec.examples.is_empty() {
        let _ = writeln!(out, "Examples:");
        for example in spec.examples {
            let _ = writeln!(out, "  {example}");
        }
        let _ = writeln!(out);
    }

    out
}

pub fn render_top_level_help(bin_name: &str, root_spec: &CommandSpec) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", root_spec.description);
    let _ = writeln!(out);

    let _ = writeln!(out, "Usage: {bin_name} [OPTIONS] <COMMAND>");
    let _ = writeln!(out);

    let _ = writeln!(out, "Global options:");
    for opt in root_spec.options {
        if !opt.hidden {
            render_option(&mut out, opt);
            let _ = writeln!(out);
        }
    }

    let _ = writeln!(out, "Commands:");
    for sub in root_spec.subcommands {
        let _ = writeln!(out, "  {}", sub.name);
        let _ = writeln!(out, "      Usage: {bin_name} {} [OPTIONS]{}", sub.name, positional_usage(sub.positionals));
        let _ = writeln!(out, "      {}", sub.description);
        let _ = writeln!(out, "      (inherits common options)");
        let _ = writeln!(out);

        if !sub.positionals.is_empty() {
            let _ = writeln!(out, "      Arguments:");
            for pos in sub.positionals {
                render_nested_positional(&mut out, pos);
                let _ = writeln!(out);
            }
        }

        render_nested_option_groups(&mut out, sub.options);
    }

    out
}

fn positional_usage(positionals: &[PositionalSpec]) -> String {
    let mut s = String::new();
    for pos in positionals {
        s.push(' ');
        if pos.required {
            if pos.repeatable {
                let _ = write!(s, "<{}>...", pos.name);
            } else {
                let _ = write!(s, "<{}>", pos.name);
            }
        } else if pos.repeatable {
            let _ = write!(s, "[{}]...", pos.name);
        } else {
            let _ = write!(s, "[{}]", pos.name);
        }
    }
    s
}

fn render_usage(out: &mut String, bin_name: &str, spec: &CommandSpec) {
    let _ = write!(out, "Usage: {bin_name} {} [OPTIONS]", spec.name);
    for pos in spec.positionals {
        let _ = write!(out, " ");
        if pos.required {
            if pos.repeatable {
                let _ = write!(out, "<{}>...", pos.name);
            } else {
                let _ = write!(out, "<{}>", pos.name);
            }
        } else if pos.repeatable {
            let _ = write!(out, "[{}]...", pos.name);
        } else {
            let _ = write!(out, "[{}]", pos.name);
        }
    }
    let _ = writeln!(out);
}

fn render_option_groups(out: &mut String, options: &[OptionSpec], _nested: bool) {
    let mut headings: Vec<Option<&'static str>> = Vec::new();
    for opt in options {
        if opt.hidden {
            continue;
        }
        if !headings.contains(&opt.heading) {
            headings.push(opt.heading);
        }
    }

    for heading in headings {
        let heading_title = heading.unwrap_or("Options");
        let _ = writeln!(out, "{heading_title}:");
        for opt in options {
            if opt.hidden || opt.heading != heading {
                continue;
            }
            render_option(out, opt);
            let _ = writeln!(out);
        }
    }
}

fn render_nested_option_groups(out: &mut String, options: &[OptionSpec]) {
    let mut headings: Vec<Option<&'static str>> = Vec::new();
    for opt in options {
        if opt.hidden {
            continue;
        }
        if !headings.contains(&opt.heading) {
            headings.push(opt.heading);
        }
    }

    for heading in headings {
        let heading_title = heading.unwrap_or("Options");
        let _ = writeln!(out, "      {heading_title}:");
        for opt in options {
            if opt.hidden || opt.heading != heading {
                continue;
            }
            render_nested_option(out, opt);
            let _ = writeln!(out);
        }
    }
}

fn render_option(out: &mut String, opt: &OptionSpec) {
    let mut flag_part = String::new();
    if let Some(short) = opt.short {
        let _ = write!(flag_part, "  -{short}, --{}", opt.long);
    } else {
        let _ = write!(flag_part, "      --{}", opt.long);
    }
    if let Some(val_name) = opt.value_name {
        let _ = write!(flag_part, " <{val_name}>");
    }

    let _ = writeln!(out, "{flag_part}");
    let _ = writeln!(out, "          {}", opt.help);

    if opt.default.is_some() || opt.env.is_some() || opt.required || opt.repeatable {
        let mut meta = Vec::new();
        if let Some(env) = opt.env {
            meta.push(format!("[env: {env}=]"));
        }
        if let Some(default) = opt.default {
            meta.push(format!("[default: {default}]"));
        }
        if opt.required {
            meta.push("[required]".to_string());
        }
        if opt.repeatable {
            meta.push("[repeatable]".to_string());
        }
        let _ = writeln!(out, "          {}", meta.join(" "));
    }
}

fn render_nested_option(out: &mut String, opt: &OptionSpec) {
    let mut flag_part = String::new();
    if let Some(short) = opt.short {
        let _ = write!(flag_part, "        -{short}, --{}", opt.long);
    } else {
        let _ = write!(flag_part, "            --{}", opt.long);
    }
    if let Some(val_name) = opt.value_name {
        let _ = write!(flag_part, " <{val_name}>");
    }

    let _ = writeln!(out, "{flag_part}");
    let _ = writeln!(out, "                {}", opt.help);

    if opt.default.is_some() || opt.env.is_some() || opt.required || opt.repeatable {
        let mut meta = Vec::new();
        if let Some(env) = opt.env {
            meta.push(format!("[env: {env}=]"));
        }
        if let Some(default) = opt.default {
            meta.push(format!("[default: {default}]"));
        }
        if opt.required {
            meta.push("[required]".to_string());
        }
        if opt.repeatable {
            meta.push("[repeatable]".to_string());
        }
        let _ = writeln!(out, "                {}", meta.join(" "));
    }
}

fn render_positional(out: &mut String, pos: &PositionalSpec) {
    let name_part = if pos.required {
        if pos.repeatable {
            format!("  <{}>...", pos.name)
        } else {
            format!("  <{}>", pos.name)
        }
    } else if pos.repeatable {
        format!("  [{}]...", pos.name)
    } else {
        format!("  [{}]", pos.name)
    };
    let _ = writeln!(out, "{name_part}");
    let _ = writeln!(out, "          {}", pos.help);
}

fn render_nested_positional(out: &mut String, pos: &PositionalSpec) {
    let name_part = if pos.required {
        if pos.repeatable {
            format!("        <{}>...", pos.name)
        } else {
            format!("        <{}>", pos.name)
        }
    } else if pos.repeatable {
        format!("        [{}]...", pos.name)
    } else {
        format!("        [{}]", pos.name)
    };
    let _ = writeln!(out, "{name_part}");
    let _ = writeln!(out, "                {}", pos.help);
}
