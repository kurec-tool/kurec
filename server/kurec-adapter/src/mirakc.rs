use kurec_interface::KurecConfig;

pub struct MirakcAdapter {
    pub config: KurecConfig,
}

impl MirakcAdapter {
    pub fn new(config: KurecConfig) -> Self {
        Self { config }
    }
}
