use super::VisualHexEnv;
use rair_core::Core;

pub struct VisualHexWithoutEnv {
    inner: VisualHexEnv,
}

impl VisualHexWithoutEnv {
    pub fn new(core: &mut Core) -> Self {
        Self {
            inner: VisualHexEnv::new(core),
        }
    }
    pub fn get_env(&mut self, core: &mut Core) -> &VisualHexEnv {
        self.inner.get_env(core)
    }
}
