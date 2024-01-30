mod de;

#[proc_macro_derive(Deserialize, attributes(toml))]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);
    de::expand(&mut input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
