use sp_core::H160;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::boxed::Box;
use crate::Application;

pub type AppRegistry = BTreeMap<H160, Box<dyn Application>>;

pub fn make_registry() -> AppRegistry {
	BTreeMap::new()
}
