// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use config::config::ConfigControl;

pub struct ControlListenBuilder;
pub struct ControlListen {
    config_control: ConfigControl
}

impl ControlListenBuilder {
    pub fn new(config_control: ConfigControl) -> ControlListen {
        ControlListen {
            config_control: config_control
        }
    }
}

impl ControlListen {
    pub fn run(&self) {
        // TODO
    }
}
