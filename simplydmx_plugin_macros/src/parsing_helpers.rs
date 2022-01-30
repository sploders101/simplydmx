use syn::{
    punctuated::Punctuated,
    Expr,
    Token,
    Lit,
    LitStr,
};

pub fn get_comma_delimited_strings(input: Punctuated<Expr, Token![,]>, error_msg: &str) -> Vec<LitStr> {
    let strings = Vec::new();

    for string in input.iter() {
		if let Expr::Lit(string) = string {
			if let Lit::Str(string) = string.lit {
				strings.push(string);
			} else {
				panic!("{}", error_msg);
			}
		} else {
			panic!("{}", error_msg);
		}
	}

    return strings;
}
