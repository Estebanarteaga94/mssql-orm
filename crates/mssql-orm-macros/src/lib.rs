use proc_macro::TokenStream;

#[proc_macro_derive(Entity, attributes(orm))]
pub fn derive_entity(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(DbContext, attributes(orm))]
pub fn derive_db_context(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(Insertable, attributes(orm))]
pub fn derive_insertable(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(Changeset, attributes(orm))]
pub fn derive_changeset(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
