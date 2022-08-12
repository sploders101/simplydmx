use std::{
	ops::Deref,
	sync::Arc,
	any::TypeId,
	marker::PhantomData,
};

use super::portable_message::PortableMessage;

/// Holds ownership of an arc to allow access to its typed data via deref
/// without de-allocating
pub struct ArcPortable<T: 'static>(Arc<Box<dyn PortableMessage>>, PhantomData<T>);

impl<T: 'static> ArcPortable<T> {

	/// Creates a new `ArcPortable` from an `Arc` containing an `Any` value.
	pub fn new(any_arc: Arc<Box<dyn PortableMessage>>) -> Option<ArcPortable<T>> {
		if (**any_arc).type_id() == TypeId::of::<T>() {
			return Some(ArcPortable(any_arc, PhantomData));
		} else {
			return None;
		}
	}

}

impl<T: 'static> Deref for ArcPortable<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		// Type checking is done during instantiation, so we can just cast the existing pointer
		return unsafe { &*(&**self.0 as *const dyn PortableMessage as *const T) };
	}
}
