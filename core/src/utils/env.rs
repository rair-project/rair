//! commands for handling environment variables.

use crate::core::Core;
use crate::helper::{error_msg, expect, expect_range, help, is_color, str_to_num, Cmd};
use rair_env::EnvData;
use std::io::Write;
use yansi::Paint;
#[derive(Default)]
pub struct Environment;

impl Environment {
    pub fn new() -> Self {
        Self
    }
    fn iterate(core: &mut Core) {
        let env = core.env.clone();
        for (k, v) in env.read().iter() {
            match v {
                EnvData::Bool(b) => writeln!(core.stdout, "{k} = {b}").unwrap(),
                EnvData::I64(i) => writeln!(core.stdout, "{k} = {i}").unwrap(),
                EnvData::U64(u) => writeln!(core.stdout, "{k} = 0x{u:x}").unwrap(),
                EnvData::Str(s) => writeln!(core.stdout, "{k} = {s}").unwrap(),
                EnvData::Color(r, g, b) => {
                    let color = format!("#{r:02x}{g:02x}{b:02x}");
                    writeln!(core.stdout, "{} = {}", k, color.rgb(r, g, b)).unwrap();
                }
            }
        }
    }
    fn set(core: &mut Core, key: &str, value: &str) {
        let env = core.env.clone();
        let mut res = Ok(());
        if env.read().is_bool(key) {
            let v_str = value.to_ascii_lowercase();
            let value = match v_str.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    let message = format!("Expected `true` or `false`, found `{value}`.");
                    return error_msg(core, "Failed to set variable.", &message);
                }
            };
            res = env.write().set_bool(key, value, core);
        } else if env.read().is_i64(key) {
            let value = match value.parse::<i64>() {
                Ok(value) => value,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            res = env.write().set_i64(key, value, core);
        } else if env.read().is_u64(key) {
            let value = match str_to_num(value) {
                Ok(value) => value,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            res = env.write().set_u64(key, value, core);
        } else if env.read().is_str(key) {
            res = env.write().set_str(key, value, core);
        } else if env.read().is_color(key) {
            if value.len() != 7 || !value.starts_with('#') {
                let message = format!("Expected color code, found `{value}`.");
                return error_msg(core, "Failed to set variable.", &message);
            }
            let r = match u8::from_str_radix(&value[1..3], 16) {
                Ok(c) => c,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            let g = match u8::from_str_radix(&value[3..5], 16) {
                Ok(c) => c,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            let b = match u8::from_str_radix(&value[5..], 16) {
                Ok(c) => c,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            res = env.write().set_color(key, (r, g, b), core);
        }
        if let Err(e) = res {
            error_msg(core, "Failed to set variable.", &e.to_string());
        }
    }
    fn display(core: &mut Core, key: &str) {
        let env = core.env.read();
        let Some(data) = env.get(key) else {
            drop(env);
            let message = format!("Variable `{key}` doesn't exist.");
            return error_msg(core, "Failed to display variable.", &message);
        };
        match data {
            EnvData::Bool(b) => writeln!(core.stdout, "{b}").unwrap(),
            EnvData::I64(i) => writeln!(core.stdout, "{i}").unwrap(),
            EnvData::U64(u) => writeln!(core.stdout, "0x{u:x}").unwrap(),
            EnvData::Str(s) => writeln!(core.stdout, "{s}").unwrap(),
            EnvData::Color(r, g, b) => {
                let color = format!("#{r:02x}{g:02x}{b:02x}");
                writeln!(core.stdout, "{}", color.rgb(r, g, b)).unwrap();
            }
        }
    }
}

impl Cmd for Environment {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() > 3 {
            expect_range(core, args.len() as u64, 0, 3);
        } else if args.is_empty() {
            Self::iterate(core);
        } else if args.len() == 1 {
            let args: Vec<&str> = args[0].split('=').collect();
            if args.len() == 2 {
                Self::set(core, args[0].trim(), args[1].trim());
            } else {
                Self::display(core, args[0]);
            }
        } else if args.len() == 2 {
            // either args[0] ends with = or args[1] starts with = but not both!
            if args[0].ends_with('=') ^ args[1].starts_with('=') {
                let key = args[0].split('=').next().unwrap().trim();
                let value = args[1].split('=').last().unwrap().trim();
                Self::set(core, key, value);
            } else {
                error_msg(core, "Failed to set variable.", "Expected `=`.");
            }
        } else if args.len() == 3 {
            if args[1] == "=" {
                Self::set(core, &args[0], &args[2]);
            } else {
                let message = format!("Expected `=` found `{}`.", args[1]);
                error_msg(core, "Failed to set variable.", &message);
            }
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "environment",
            "e",
            vec![
                ("", "List all environment variables."),
                ("[var]", "Display the value of [var] environment variables."),
                ("[var]=[value]", "Set [var] to be [value]"),
            ],
        );
    }
}

#[derive(Default)]
pub struct EnvironmentReset;

impl EnvironmentReset {
    pub fn new() -> Self {
        Self
    }
}

impl Cmd for EnvironmentReset {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        let env = core.env.clone();
        let res = env.write().reset(&args[0], core);
        if let Err(e) = res {
            error_msg(core, "Failed to reset variable.", &e.to_string());
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "environmentReset",
            "er",
            vec![("[var]", "Reset [var] environment variable.")],
        );
    }
}

#[derive(Default)]
pub struct EnvironmentHelp;

impl EnvironmentHelp {
    pub fn new(core: &mut Core) -> Self {
        let env = core.env.clone();
        env.write()
            .add_str_with_cb(
                "environmentHelp.envColor",
                "color.6",
                "Color used in the environment variable",
                core,
                is_color,
            )
            .unwrap();
        Self
    }
}

impl Cmd for EnvironmentHelp {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        let env = core.env.read();
        let res = env.get_help(&args[0]);
        if let Some(help) = res {
            let color = env.get_str("environmentHelp.envColor").unwrap();
            let (r, g, b) = env.get_color(color).unwrap();
            writeln!(core.stdout, "{}:\t{}", &args[0].rgb(r, g, b), help).unwrap();
        } else {
            drop(env);
            error_msg(core, "Failed to display help.", "Variable Not found");
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            "environmentHelp",
            "eh",
            vec![("[var]", "Print help for [var] environment variable.")],
        );
    }
}

#[cfg(test)]
mod test_env {
    extern crate alloc;

    use super::*;
    use crate::writer::*;
    use alloc::sync::Arc;
    use rair_env::Environment as Env;
    #[test]
    fn test_help() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let er = EnvironmentReset::new();
        let env = Environment::new();
        er.help(&mut core);
        env.help(&mut core);
        core.help("eh");
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [environmentReset | er]\n\
             Usage:\n\
             er [var]\tReset [var] environment variable.\n\
             Commands: [environment | e]\n\
             Usage:\n\
             e\tList all environment variables.\n\
             e [var]\tDisplay the value of [var] environment variables.\n\
             e [var]=[value]\tSet [var] to be [value]\n\
             Commands: [environmentHelp | eh]\n\
             Usage:\n\
             eh [var]\tPrint help for [var] environment variable.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_env_help() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run("eh", &["environmentHelp.envColor".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "environmentHelp.envColor:\tColor used in the environment variable\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run("eh", &["doesnt.exist".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to display help.\nVariable Not found\n"
        );
    }
    #[test]
    fn test_env_reset() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut er = EnvironmentReset::new();
        let env = core.env.clone();
        let (r, g, b) = env.read().get_color("color.1").unwrap();
        env.write()
            .set_color("color.1", (r + 1, g + 1, b + 1), &mut core)
            .unwrap();
        er.run(&mut core, &["color.1".to_owned()]);
        let (r2, g2, b2) = env.read().get_color("color.1").unwrap();
        assert_eq!(r, r2);
        assert_eq!(g, g2);
        assert_eq!(b, b2);
    }
    #[test]
    fn test_env_reset_err() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut er = EnvironmentReset::new();
        er.run(&mut core, &["doest.exist".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to reset variable.\nEnvironment variable not found.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        er.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected 1 argument(s), found 0.\n"
        );
    }
    fn get_good_core() -> Core {
        let mut core = Core::new_no_colors();
        core.env = Arc::default();
        core.env.write().add_bool("b", false, "").unwrap();
        core.env.write().add_u64("u", 500, "").unwrap();
        core.env.write().add_i64("i", -500, "").unwrap();
        core.env.write().add_str("s", "hello world", "").unwrap();
        core.env
            .write()
            .add_color("c", (0xff, 0xee, 0xdd), "")
            .unwrap();
        core
    }
    #[test]
    fn test_env_0() {
        let mut core = get_good_core();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();

        let mut env = Environment::new();
        env.run(&mut core, &[]);
        let s = core.stdout.utf8_string().unwrap();
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(s.len(), 57);
        assert!(s.contains("i = -500\n"));
        assert!(s.contains("u = 0x1f4\n"));
        assert!(s.contains("s = hello world\n"));
        assert!(s.contains("b = false\n"));
        assert!(s.contains("c = #ffeedd\n"));
    }
    #[test]
    fn test_env_1() {
        let mut core = get_good_core();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(&mut core, &["b".to_owned()]);
        env.run(&mut core, &["u".to_owned()]);
        env.run(&mut core, &["i".to_owned()]);
        env.run(&mut core, &["s".to_owned()]);
        env.run(&mut core, &["c".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "false\n0x1f4\n-500\nhello world\n#ffeedd\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["b  = true ".to_owned()]);
        env.run(&mut core, &["u= 0x5".to_owned()]);
        env.run(&mut core, &["i=-1".to_owned()]);
        env.run(&mut core, &["s=happy birthday".to_owned()]);
        env.run(&mut core, &["c=#aaaaaa".to_owned()]);
        env.run(&mut core, &["b".to_owned()]);
        env.run(&mut core, &["u".to_owned()]);
        env.run(&mut core, &["i".to_owned()]);
        env.run(&mut core, &["s".to_owned()]);
        env.run(&mut core, &["c".to_owned()]);
        env.run(&mut core, &["b  = false".to_owned()]);
        env.run(&mut core, &["b".to_owned()]);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "true\n0x5\n-1\nhappy birthday\n#aaaaaa\nfalse\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_env_2() {
        let mut core = get_good_core();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(&mut core, &["b  =".to_owned(), "true ".to_owned()]);
        env.run(&mut core, &["u".to_owned(), "= 0x5".to_owned()]);
        env.run(&mut core, &["b".to_owned()]);
        env.run(&mut core, &["u".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "true\n0x5\n");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_env_3() {
        let mut core = get_good_core();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(
            &mut core,
            &["b".to_owned(), "=".to_owned(), "true".to_owned()],
        );
        env.run(&mut core, &["b".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "true\n");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_env_error() {
        let mut core = Core::new_no_colors();
        core.env.write().add_bool("b", false, "").unwrap();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(
            &mut core,
            &[
                "b".to_owned(),
                "=".to_owned(),
                "true".to_owned(),
                "extra".to_owned(),
            ],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Arguments Error: Expected between 0 and 3 arguments, found 4.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(
            &mut core,
            &["b".to_owned(), "true".to_owned(), "extra".to_owned()],
        );
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\nExpected `=` found `true`.\n"
        );
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["b".to_owned(), "true".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\nExpected `=`.\n"
        );
    }
    #[test]
    fn test_display_error() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(&mut core, &["b".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to display variable.\nVariable `b` doesn't exist.\n"
        );
    }

    fn always_false(_: &str, value: bool, _: &Env<Core>, _: &mut Core) -> bool {
        !value
    }

    #[test]
    fn test_set_error() {
        let mut core = Core::new_no_colors();
        let env = core.env.clone();
        env.write()
            .add_bool_with_cb("b", false, "", &mut core, always_false)
            .unwrap();
        env.write().add_u64("u", 500, "").unwrap();
        env.write().add_i64("i", -500, "").unwrap();
        env.write().add_str("s", "hi", "").unwrap();
        env.write().add_color("c", (0xee, 0xee, 0xee), "").unwrap();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(&mut core, &["b=no".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\nExpected `true` or `false`, found `no`.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["b=true".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\nCall back failed.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["i=x5".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["u=x5".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["c=5".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\nExpected color code, found `5`.\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["c=#1x2233".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["c=#11x233".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\ninvalid digit found in string\n"
        );

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["c=#11223x".to_owned()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Failed to set variable.\ninvalid digit found in string\n"
        );
    }
}
