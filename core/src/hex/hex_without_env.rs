use super::HexEnv;
use crate::Core;

pub struct HexWithoutEnv {
    inner: HexEnv,
}

impl HexWithoutEnv {
    pub fn get_env(&mut self, core: &mut Core) -> &HexEnv {
        self.inner.get_env(core)
    }
    pub fn new(core: &mut Core) -> Self {
        Self {
            inner: HexEnv::new(core),
        }
    }
}
