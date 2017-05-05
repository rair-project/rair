/*
 *  Copyright (C) 2017  Ahmed Abd El Mawgood
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::collections::HashMap;

type RConfigCb<T> = Box<for<'a> Fn(&'a mut T, &'a str, &'a str) + Send>;

struct RConfigEntry<T> {
    value: String,
    setter_cb: Option<RConfigCb<T>>,
    getter_cb: Option<RConfigCb<T>>,
}

pub struct RConfig<T: Sized> {
    hm: HashMap<String, RConfigEntry<T>>,
}

impl<T> RConfigEntry<T> {
    pub fn new(value: String) -> RConfigEntry<T> {
        RConfigEntry {
            value: value,
            setter_cb: None,
            getter_cb: None,
        }
    }
}

impl<T> RConfig<T> {
    pub fn new() -> RConfig<T> {
        RConfig { hm: HashMap::new() }
    }

    pub fn new_str(&mut self, name: &str, value: &str) -> Result<(), &'static str> {
        if self.hm.get(name).is_some() {
            return Err("Name already exists");
        }
        let entry = RConfigEntry::new(value.to_owned());
        self.hm.insert(name.to_owned(), entry);
        Ok(())
    }

    pub fn set_setter_cb(&mut self, name: &str, setter: RConfigCb<T>) -> Result<(), &'static str> {
        let mut entry = match self.hm.get_mut(name) {
            Some(x) => x,
            None => return Err("Entry doesn't Exist"),
        };
        entry.setter_cb = Some(setter);
        Ok(())
    }

    pub fn set_getter_cb(&mut self, name: &str, getter: RConfigCb<T>) -> Result<(), &'static str> {
        let mut entry = match self.hm.get_mut(name) {
            Some(x) => x,
            None => return Err("Entry doesn't Exist"),
        };
        entry.getter_cb = Some(getter);
        Ok(())
    }

    pub fn set(&mut self, user: &mut T, name: &str, value: &str) {
        if self.hm.get(name).is_none() {
            self.new_str(name, value).unwrap();
        } else {
            let mut entry = self.hm.get_mut(name).unwrap();
            entry.value = value.to_owned();
            if entry.setter_cb.is_some() {
                (entry.setter_cb.as_ref().unwrap())(user, name, value);
            }
        }
    }

    pub fn new_bool(&mut self, name: &str, value: bool) -> Result<(), &'static str> {
        let str_value = if value {
            "true"
        } else {
            "false"
        };
        self.new_str(name, str_value)
    }

    pub fn set_bool(&mut self, user: &mut T, name: &str, value: bool) {
        let str_value = if value {
            "true" 
        } else {
            "false"
        };
        self.set(user, name, str_value);
    }

    pub fn get_bool(&mut self, user: &mut T, name: &str) -> Result<bool, &'static str> {
        match self.hm.get(name) {
            Some(entry) => {
                match &*entry.value {
                    "true" => {
                        if entry.getter_cb.is_some() {
                            (entry.getter_cb.as_ref().unwrap())(user, name, "true");
                        }
                        Ok(true)
                    }
                    "false" => {
                        if entry.getter_cb.is_some() {
                            (entry.getter_cb.as_ref().unwrap())(user, name, "false")
                        }
                        Ok(false)
                    }
                    _ => Err("Failed parsing as bool"),
                }
            }
            None =>Err("Not found"),
        }
    }

    pub fn get(&self, user: &mut T, name: &str) -> Option<String> {
        match self.hm.get(name) {
            Some(entry) => {
                if entry.setter_cb.is_some() {
                    (entry.setter_cb.as_ref().unwrap())(user, name, &entry.value);
                }
                Some(entry.value.to_owned())
            }
            None => None,
        }
    }

    pub fn new_i64(&mut self, name: &str, value: i64) -> Result<(), &'static str> {
        if self.hm.get(name).is_some() {
            return Err("Name already exists");
        }
        self.new_str(name, &value.to_string())
    }

    pub fn set_i64(&mut self, user: &mut T, name: &str, value: i64) {
        self.set(user, name, &value.to_string());
    }

    pub fn get_i64(&self, user: &mut T, name: &str) -> Result<i64, String> {
        match self.hm.get(name) {
            Some(entry) => {
                match entry.value.parse() {
                    Ok(x) => {
                        if entry.getter_cb.is_some() {
                            (entry.getter_cb.as_ref().unwrap())(user, name, &entry.value);
                        }
                        Ok(x)
                    }
                    Err(y) => Err(y.to_string()),
                }
            }
            None => Err("Not found".to_owned()),
        }
    }

    pub fn new_u64(&mut self, name: &str, value: u64) -> Result<(), &'static str> {
        if self.hm.get(name).is_some() {
            return Err("Name already exists");
        }
        self.new_str(name, &value.to_string())
    }

    pub fn set_u64(&mut self, user: &mut T, name: &str, value: u64) {
        self.set(user, name, &value.to_string());
    }

    pub fn get_u64(&self, user: &mut T, name: &str) -> Result<u64, String> {
        match self.hm.get(name) {
            Some(entry) => {
                match entry.value.parse() {
                    Ok(x) => {
                        if entry.getter_cb.is_some() {
                            (entry.getter_cb.as_ref().unwrap())(user, name, &entry.value);
                        }
                        Ok(x)
                    }
                    Err(y) => Err(y.to_string()),
                }
            }
            None => Err("Not found".to_owned()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    struct User {
        cb_str: bool,
        cb_i64: bool,
        cb_u64: bool,
        cb_bool: bool,
    }
    struct Everything {
        user:User,
        config: RConfig<User>,
    }
    impl User {
        fn new() -> User{
            User {
                cb_str: false,
                cb_i64: false,
                cb_u64: false,
                cb_bool: false,
            }
        }
    }
    #[test]
    fn test_all_no_cbs() {
        let mut e = Everything{user:User::new(), config:RConfig::new()};
        let ref mut user = e.user;
        let ref mut config = e.config;
        config.new_str("new name", "new value").unwrap();
        config.new_bool("new bool", true).unwrap();
        config.new_i64("new i64", -172172).unwrap();
        config.new_u64("new u64", 172172).unwrap();

        assert!(config.get(user, "new name").unwrap() == "new value");
        assert!(config.get_bool(user, "new bool").unwrap() == true);
        assert!(config.get_i64(user, "new i64").unwrap() == -172172);
        assert!(config.get_u64(user, "new u64").unwrap() == 172172);
    }
}
