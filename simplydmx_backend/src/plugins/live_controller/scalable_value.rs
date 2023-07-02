use midly::num::u7;

#[derive(Debug, Clone, Copy)]
/// Represents a value of varying precision used with control surfaces. This value should
/// be treated the same as a low-precision floating-point number. In other words, it should
/// be treated as a rough estimate of the intended value. Built-in comparison logic is fuzzy
/// and subject to change.
///
/// This value can be converted to any of its contained types, and auto-scaling will occur.
/// The contents are treated as a percentage of the inner type's max value.
///
/// ```rust
/// # use simplydmx_lib::plugins::live_controller::scalable_value::ScalableValue;
/// # use midly::num::u7;
/// #
/// let u7_max = ScalableValue::U7(127.into());
/// let u8_max = ScalableValue::U8(255u8);
/// let u16_max = ScalableValue::U16(65535u16);
/// let u7_min = ScalableValue::U7(0.into());
/// let u8_min = ScalableValue::U8(0u8);
/// let u16_min = ScalableValue::U16(0u16);
///
/// assert_eq!(u7_max.as_u7(), u7::max_value());
/// assert_eq!(u7_max.as_u8(), u8::MAX);
/// assert_eq!(u7_max.as_u16(), u16::MAX);
/// assert_eq!(u8_max.as_u7(), u7::max_value());
/// assert_eq!(u8_max.as_u8(), u8::MAX);
/// assert_eq!(u8_max.as_u16(), u16::MAX);
/// assert_eq!(u16_max.as_u7(), u7::max_value());
/// assert_eq!(u16_max.as_u8(), u8::MAX);
/// assert_eq!(u16_max.as_u16(), u16::MAX);
/// assert_eq!(u7_min.as_u7(), u7::from(0));
/// assert_eq!(u7_min.as_u8(), u8::MIN);
/// assert_eq!(u7_min.as_u16(), u16::MIN);
/// assert_eq!(u8_min.as_u7(), u7::from(0));
/// assert_eq!(u8_min.as_u8(), u8::MIN);
/// assert_eq!(u8_min.as_u16(), u16::MIN);
/// assert_eq!(u16_min.as_u7(), u7::from(0));
/// assert_eq!(u16_min.as_u8(), u8::MIN);
/// assert_eq!(u16_min.as_u16(), u16::MIN);
/// ```
pub enum ScalableValue {
	U7(u7),
	U8(u8),
	U16(u16),
}
impl ScalableValue {
	pub fn normalize(&self, other: &Self) -> (Self, Self) {
		return match (self, other) {
			(Self::U7(_), Self::U7(_)) => (*self, *other),
			(Self::U7(_), Self::U8(_)) => (Self::U8(self.clone().into()), *other),
			(Self::U7(_), Self::U16(_)) => (Self::U16(self.clone().into()), *other),
			(Self::U8(_), Self::U8(_)) => (*self, *other),
			(Self::U8(_), Self::U7(_)) => (*self, Self::U8(other.clone().into())),
			(Self::U8(_), Self::U16(_)) => (Self::U16(self.clone().into()), *other),
			(Self::U16(_), Self::U16(_)) => (*self, *other),
			(Self::U16(_), Self::U7(_)) => (*self, Self::U16(other.clone().into())),
			(Self::U16(_), Self::U8(_)) => (*self, Self::U16(other.clone().into())),
		};
	}
	pub fn as_u7(self) -> u7 {
		self.into()
	}
	pub fn as_u8(self) -> u8 {
		self.into()
	}
	pub fn as_u16(self) -> u16 {
		self.into()
	}
}
impl PartialEq for ScalableValue {
	fn eq(&self, other: &Self) -> bool {
		let (a, b) = self.normalize(other);
		return match (a, b) {
			(Self::U7(a), Self::U7(b)) => a.eq(&b),
			(Self::U8(a), Self::U8(b)) => a.eq(&b),
			(Self::U16(a), Self::U16(b)) => a.eq(&b),
			_ => unreachable!(),
		};
	}
}
impl Eq for ScalableValue {}
impl PartialOrd for ScalableValue {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		let (a, b) = self.normalize(other);
		match (a, b) {
			(Self::U7(a), Self::U7(b)) => a.partial_cmp(&b),
			(Self::U8(a), Self::U8(b)) => a.partial_cmp(&b),
			(Self::U16(a), Self::U16(b)) => a.partial_cmp(&b),
			_ => unreachable!(),
		}
	}
}
impl Ord for ScalableValue {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let (a, b) = self.normalize(other);
		match (a, b) {
			(Self::U7(a), Self::U7(b)) => a.cmp(&b),
			(Self::U8(a), Self::U8(b)) => a.cmp(&b),
			(Self::U16(a), Self::U16(b)) => a.cmp(&b),
			_ => unreachable!(),
		}
	}
}
impl Into<u7> for ScalableValue {
	fn into(self) -> u7 {
		// These values are approximations since 7 is not a multiple of 8.
		// The semantics of integer division are required here since, for example,
		// `65535 / 514 = 127.5`. Flooring is necessary for this approximation.
		// `255 / 127 = 2.007874015748`
		// `65535 / 127 = 516.02362204724`
		match self {
			Self::U7(num) => num,
			Self::U8(num) => (num / 2).into(),
			Self::U16(num) => ((num / 514) as u8).into(),
		}
	}
}
impl Into<u8> for ScalableValue {
	fn into(self) -> u8 {
		match self {
			Self::U7(num) => (num.as_int() as f32 * 2.007874015748) as u8,
			Self::U8(num) => num,
			Self::U16(num) => (num / 257) as u8,
		}
	}
}
impl Into<u16> for ScalableValue {
	fn into(self) -> u16 {
		match self {
			Self::U7(num) => (num.as_int() as f32 * 516.02362204724) as u16,
			Self::U8(num) => num as u16 * 257,
			Self::U16(num) => num,
		}
	}
}
