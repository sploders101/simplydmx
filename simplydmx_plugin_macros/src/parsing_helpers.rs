use syn::{
	punctuated::Punctuated,
	Expr,
	Token,
	Lit,
	LitStr,
};

pub fn get_comma_delimited_strings(input: &Punctuated<Expr, Token![,]>, error_msg: &str) -> Vec<LitStr> {
	let mut strings = Vec::new();

	for string in input.iter() {
		if let Expr::Lit(string) = string {
			if let Lit::Str(ref string) = string.lit {
				strings.push(LitStr::clone(string));
			} else {
				panic!("{}", error_msg);
			}
		} else {
			panic!("{}", error_msg);
		}
	}

	return strings;
}
