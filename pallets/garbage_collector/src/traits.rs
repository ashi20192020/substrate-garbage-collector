pub trait Cleanable {
	fn cleanable() -> bool
	where
		Self: Sized;
}

pub trait CleanableAction {
	fn cleanable_action()
	where
		Self: Sized;
}
