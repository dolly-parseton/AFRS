extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DataStruct, Fields, FieldsNamed, Ident, Lit, Meta};

#[proc_macro_derive(Searchable)]
pub fn derive_print_schema(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();
    // Build the impl
    impl_print_schema(&ast)
}

fn impl_print_schema(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let namespace = Ident::new(&format!("_IMPL_SCHEMA_FOR_{}", ast.ident), ast.ident.span());
    // let field = Ident::new(&format!("{}Field",ast.ident), ast.ident.span());
    // let schema = Ident::new(&format!("{}Schema",ast.ident), ast.ident.span());
    // let print_trait = Ident::new(&format!("{}PrintSchema",ast.ident), ast.ident.span());

    let _fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    let builder_fields = _fields
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            let field_type = &f.ty;
            //
            let mut quote = quote! {
                #[automatically_derived]
                _searchable::Field {
                    field_name: stringify!(#field_name).to_string(),
                    field_type: stringify!(#field_type).to_string(),
                    doc: None,
                },
            };
            if let Some(a) = get_field_attr(&f) {
                if let Ok(meta) = a.parse_meta() {
                    if let Meta::NameValue(m) = meta {
                        if m.path.is_ident("doc") {
                            if let Lit::Str(lit_str) = &m.lit {
                                let mut doc_string = lit_str.value();
                                if &[doc_string.as_bytes()[0]] == " ".as_bytes() {
                                    doc_string.remove(0);
                                }
                                quote = quote! {
                                    #[automatically_derived]
                                    _searchable::Field {
                                        field_name: stringify!(#field_name).to_string(),
                                        field_type: stringify!(#field_type).to_string(),
                                        doc: Some(String::from(#doc_string)),
                                    },
                                }
                            }
                        }
                    }
                }
            }
            quote
        })
        .collect::<TokenStream2>();
    let implmentation = quote! {
        #[allow(non_upper_case_globals,unused_attributes,unused_qualifiers)]
        const #namespace: () = {
            extern crate searchable as _searchable;
            #[automatically_derived]
            impl _searchable::Searchable for #name {
                fn get_value(&self) -> Option<&[u8]> {
                    _searchable::Searchable {
                        name: stringify!(#name).to_string(),
                        fields: vec![#builder_fields],
                    }
                }
            }
        };
    };
    implmentation.into()
}

fn get_field_attr(field: &syn::Field) -> Option<&syn::Attribute> {
    let mut attrs: Vec<&syn::Attribute> = field
        .attrs
        .iter()
        .filter(|at| at.path.is_ident("doc"))
        .collect();
    attrs.reverse();
    attrs.pop()
}
