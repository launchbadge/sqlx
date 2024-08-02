use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr,
};

/// Macro input `query!()` and `query_file!()`
pub struct DatabaseMacroInput {
    pub(super) env: String,
}

impl Parse for DatabaseMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut env = None;

        let mut expect_comma = false;
        while !input.is_empty() {
            if expect_comma {
                let _ = input.parse::<syn::token::Comma>()?;
            }

            let key: Ident = input.parse()?;

            let _ = input.parse::<syn::token::Eq>()?;

            if key == "env" {
                env = Some(input.parse::<LitStr>()?.value());
            } else {
                let message = format!("unexpected input key: {key}");
                return Err(syn::Error::new_spanned(key, message));
            }

            expect_comma = true;
        }

        let env = env.ok_or_else(|| input.error("expected `env` key"))?;

        Ok(DatabaseMacroInput { env })
    }
}
