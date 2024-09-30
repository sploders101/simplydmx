use crate::utils::{id_alloc::Id, smallmap::SmallMap};

use super::Submaster;

/// Contains contextual details like timing information for FX
pub struct ShaderContext<'a> {
	submasters: &'a SmallMap<(Id, Id), Submaster>,
}
