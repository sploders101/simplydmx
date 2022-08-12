use std::{
	ops::Deref,
	sync::Arc,
	any::{
		TypeId,
		Any,
	},
	marker::PhantomData,
};


/// Holds ownership of an arc to allow access to its typed data via deref
/// without de-allocating
pub struct ArcAny<T: 'static>(Arc<Box<dyn Any + Sync + Send>>, PhantomData<T>);

impl<T: 'static> ArcAny<T> {

	/// Creates a new `ArcAny` from an `Arc` containing an `Any` value.
	pub fn new(any_arc: Arc<Box<dyn Any + Sync + Send>>) -> Option<ArcAny<T>> {
		if (**any_arc).type_id() == TypeId::of::<T>() {
			return Some(ArcAny(any_arc, PhantomData));
		} else {
			return None;
		}
	}

}

impl<T: 'static> Deref for ArcAny<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		// Type checking is done during instantiation, so we can just cast the existing pointer
		return unsafe { &*(&**self.0 as *const dyn Any as *const T) };
	}
}
