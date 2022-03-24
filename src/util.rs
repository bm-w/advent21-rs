// Copyright (c) 2022 Bastiaan Marinus van de Weerd


macro_rules! mod_days {
	( $( $num:literal ),+ ) => {
		paste::paste! { $( mod [<day $num>]; )+ }
	}
}
pub(crate) use mod_days;


pub(crate) mod cast {
	// Adapted from: https://stackoverflow.com/a/60572615/316870

	#[derive(Debug, PartialEq, Eq)]
	pub(crate) enum CastError { NotFilled(usize), OverFilled }

	pub(crate) trait Cast<T, U: Default + AsMut<[T]>>: Sized + Iterator<Item = T> {
		fn cast(mut self) -> Result<U, CastError> {
			// TODO(bm-w): Unsafely use uninitialized memory to avoid `Default` bound?
			let mut out: U = U::default();
			let arr: &mut [T] = out.as_mut();
			for (i, elt) in arr.iter_mut().enumerate() {
				match self.next() {
					None => { return Err(CastError::NotFilled(i)); }
					Some(v) => { *elt = v; }
				}
			}
			if self.next().is_some() {
				return Err(CastError::OverFilled)
			}
			Ok(out)
		}
	}

	impl<T, U: Iterator<Item = T>, V: Default + AsMut<[T]>> Cast<T, V> for U {}

	#[test]
	fn test() {
		assert_eq!(&[1, 2, 3].into_iter().cast(), &Ok([1, 2, 3]));
	}
}
