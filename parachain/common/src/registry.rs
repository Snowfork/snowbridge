
use crate::types::AppID;

pub enum AppName {
    PolkaETH
}

pub struct Entry {
    pub symbol: AppName,
    pub id: AppID,
}

pub static REGISTRY: &'static [Entry] = &[
    Entry {
        symbol: AppName::PolkaETH,
        id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    }
];
