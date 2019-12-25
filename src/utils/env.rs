/*
 * env.rs: commands for handling environment variables.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use core::*;
use helper::*;
use renv::EnvData;
use std::io::Write;
use yansi::Paint;

#[derive(Default)]
pub struct Environment {}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }
    fn iterate(&self, core: &mut Core) {
        let env = core.env.clone();
        for (k, v) in env.borrow().iter() {
            match v {
                EnvData::Bool(b) => writeln!(core.stdout, "{} = {}", k, b).unwrap(),
                EnvData::I64(i) => writeln!(core.stdout, "{} = {}", k, i).unwrap(),
                EnvData::U64(u) => writeln!(core.stdout, "{} = 0x{:x}", k, u).unwrap(),
                EnvData::Str(s) => writeln!(core.stdout, "{} {}", k, s).unwrap(),
                EnvData::Color(r, g, b) => {
                    let color = format!("#{:02x}{:02x}{:02x}", r, g, b);
                    writeln!(core.stdout, "{} = {}", k, Paint::rgb(r, g, b, color)).unwrap();
                }
            }
        }
    }
    fn set(&self, core: &mut Core, key: &str, value: &str) {
        let env = core.env.clone();
        if env.borrow().is_bool(key) {
            let v_str = value.to_ascii_lowercase();
            let value = match v_str.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    let message = format!("Expected `true` or `false`, found `{}`", value);
                    return error_msg(core, "Failed to set variable.", &message);
                }
            };
            if let Err(e) = env.borrow_mut().set_bool(key, value, core) {
                return error_msg(core, "Failed to set variable.", &e.to_string());
            }
        } else if env.borrow().is_i64(key) {
            let value = match i64::from_str_radix(value, 10) {
                Ok(value) => value,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            if let Err(e) = env.borrow_mut().set_i64(key, value, core) {
                return error_msg(core, "Failed to set variable.", &e.to_string());
            }
        } else if env.borrow().is_u64(key) {
            let value = match str_to_num(value) {
                Ok(value) => value,
                Err(e) => return error_msg(core, "Failed to set variable.", &e.to_string()),
            };
            if let Err(e) = env.borrow_mut().set_u64(key, value, core) {
                return error_msg(core, "Failed to set variable.", &e.to_string());
            }
        } else if env.borrow().is_str(key) {
            if let Err(e) = env.borrow_mut().set_str(key, value, core) {
                return error_msg(core, "Failed to set variable.", &e.to_string());
            }
        } else if env.borrow().is_color(key) {
            if value.len() != 7 || !value.starts_with('#') {
                let message = format!("Expected color code, found `{}`", value);
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
            if let Err(e) = env.borrow_mut().set_color(key, (r, g, b), core) {
                return error_msg(core, "Failed to set variable.", &e.to_string());
            }
        }
    }
    fn display(&self, core: &mut Core, key: &str) {
        let env = core.env.borrow();
        let data = match env.get(key) {
            Some(data) => data,
            None => {
                drop(env);
                let message = format!("variable `{}` doesn't exist.", key);
                return error_msg(core, "Failed to display variable.", &message);
            }
        };
        match data {
            EnvData::Bool(b) => writeln!(core.stdout, "{}", b).unwrap(),
            EnvData::I64(i) => writeln!(core.stdout, "{}", i).unwrap(),
            EnvData::U64(u) => writeln!(core.stdout, "0x{:x}", u).unwrap(),
            EnvData::Str(s) => writeln!(core.stdout, "{}", s).unwrap(),
            EnvData::Color(r, g, b) => {
                let color = format!("#{:02x}{:02x}{:02x}", r, g, b);
                writeln!(core.stdout, "{}", Paint::rgb(r, g, b, color)).unwrap();
            }
        }
    }
}

impl Cmd for Environment {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() > 3 {
            return expect_range(core, args.len() as u64, 0, 3);
        } else if args.is_empty() {
            self.iterate(core);
        } else if args.len() == 1 {
            let args: Vec<&str> = args[0].split('=').collect();
            if args.len() == 2 {
                self.set(core, &args[0].trim(), &args[1].trim());
            } else {
                self.display(core, &args[0]);
            }
        } else if args.len() == 2 {
            // either args[0] ends with = or args[1] starts with = but not both!
            if args[0].ends_with('=') ^ args[1].starts_with('=') {
                let key = args[0].split('=').next().unwrap().trim();
                let value = args[1].split('=').last().unwrap().trim();
                self.set(core, key, value);
            } else {
                return error_msg(core, "Failed to set variable.", &"Expected `=`.");
            }
        } else if args.len() == 3 {
            if args[1] == "=" {
                self.set(core, &args[0], &args[1]);
            } else {
                let message = format!("Expected `=` found `{}`", args[1]);
                return error_msg(core, "Failed to set variable.", &message);
            }
        }
    }
    fn help(&self, core: &mut Core) {
        help(
            core,
            &"environment",
            &"e",
            vec![
                ("", "List all environment variables."),
                ("[var]", "Display the value of [var] environment variables."),
                ("[var]=[value]", "Set [var] to be [value]"),
            ],
        );
    }
}

#[derive(Default)]
pub struct EnvironmentReset {}

impl EnvironmentReset {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for EnvironmentReset {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 1 {
            expect(core, args.len() as u64, 1);
            return;
        }
        let env = core.env.clone();
        let res = env.borrow_mut().reset(&args[0], core);
        if let Err(e) = res {
            return error_msg(core, "Failed to reset variable.", &e.to_string());
        }
        return;
    }
    fn help(&self, core: &mut Core) {
        help(core, &"environmentReset", &"er", vec![("[var]", "Reset [var] environment variable.")]);
    }
}

#[cfg(test)]
mod test_env {
    use super::*;
    use writer::*;
    #[test]
    fn test_help() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let er = EnvironmentReset::new();
        let env = Environment::new();
        er.help(&mut core);
        env.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [environmentReset | er]\n\n\
             Usage:\n\
             er [var]\tReset [var] environment variable.\n\
             Commands: [environment | e]\n\n\
             Usage:\n\
             e\tList all environment variables.\n\
             e [var]\tDisplay the value of [var] environment variables.\n\
             e [var]=[value]\tSet [var] to be [value]\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_env_reset() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut er = EnvironmentReset::new();
        let env = core.env.clone();
        let (r, g, b) = env.borrow().get_color("color.1").unwrap();
        env.borrow_mut().set_color("color.1", (r + 1, g + 1, b + 1), &mut core).unwrap();
        er.run(&mut core, &["color.1".to_string()]);
        let (r2, g2, b2) = env.borrow().get_color("color.1").unwrap();
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
        er.run(&mut core, &["doest.exist".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Failed to reset variable.\nEnvironment variable not found.\n");

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        er.run(&mut core, &[]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Arguments Error: Expected 1 argument(s), found 0.\n");
    }
    fn get_good_core() -> Core {
        let mut core = Core::new_no_colors();
        core.env = Default::default();
        core.env.borrow_mut().add_bool("b", false, "").unwrap();
        core.env.borrow_mut().add_u64("u", 500, "").unwrap();
        core.env.borrow_mut().add_i64("i", -500, "").unwrap();
        core.env.borrow_mut().add_str("s", "hello world", "").unwrap();
        core.env.borrow_mut().add_color("c", (0xff, 0xee, 0xdd), "").unwrap();
        return core;
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
        assert_eq!(s.len(), 55);
        assert!(s.contains("i = -500\n"));
        assert!(s.contains("u = 0x1f4\n"));
        assert!(s.contains("s hello world\n"));
        assert!(s.contains("b = false\n"));
        assert!(s.contains("c = #ffeedd\n"));
    }
    #[test]
    fn test_env_1() {
        let mut core = get_good_core();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(&mut core, &["b".to_string()]);
        env.run(&mut core, &["u".to_string()]);
        env.run(&mut core, &["i".to_string()]);
        env.run(&mut core, &["s".to_string()]);
        env.run(&mut core, &["c".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "false\n0x1f4\n-500\nhello world\n#ffeedd\n");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");

        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        env.run(&mut core, &["b  = true ".to_string()]);
        env.run(&mut core, &["u= 0x5".to_string()]);
        env.run(&mut core, &["i=-1".to_string()]);
        env.run(&mut core, &["s=happy birthday".to_string()]);
        env.run(&mut core, &["c=#aaaaaa".to_string()]);
        env.run(&mut core, &["b".to_string()]);
        env.run(&mut core, &["u".to_string()]);
        env.run(&mut core, &["i".to_string()]);
        env.run(&mut core, &["s".to_string()]);
        env.run(&mut core, &["c".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "true\n0x5\n-1\nhappy birthday\n#aaaaaa\n");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }

    #[test]
    fn test_env_2() {
        let mut core = get_good_core();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let mut env = Environment::new();
        env.run(&mut core, &["b  =".to_string(), "true ".to_string()]);
        env.run(&mut core, &["u".to_string(), "= 0x5".to_string()]);
        env.run(&mut core, &["b".to_string()]);
        env.run(&mut core, &["u".to_string()]);
        assert_eq!(core.stdout.utf8_string().unwrap(), "true\n0x5\n");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
}
