use rair_core::Core;
use rair_env::Environment;

pub fn one_byte(_: &str, value: &str, _: &Environment<Core>, _: &mut Core) -> bool {
    value.len() == 1
}
