use std::{collections::HashMap, sync::Mutex};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Brace,
    Expr, Ident, ItemStruct, Token, Type,
};

enum FieldValue {
    LoadStruct(StructValue),
    Expr(Expr),
}

impl ToTokens for FieldValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldValue::LoadStruct(struct_value) => struct_value.to_tokens(tokens),
            FieldValue::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

struct StructValue {
    fields: HashMap<Ident, FieldValue>,
}

impl Parse for StructValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);

        let mut fields = HashMap::new();
        while !content.is_empty() {
            let name: Ident = content.parse()?;
            content.parse::<Token![=]>()?;
            let value = if content.peek(Brace) {
                FieldValue::LoadStruct(StructValue::parse(&content)?)
            } else {
                FieldValue::Expr(content.parse()?)
            };
            fields.insert(name, value);

            if !content.peek(Token![,]) {
                break;
            } else {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(StructValue { fields })
    }
}

impl ToTokens for StructValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields_tokens = self.fields.iter().map(|(field_name, field_value)| {
            let field_tokens = match field_value {
                FieldValue::LoadStruct(struct_value) => {
                    let structs = &STRUCTS.lock().unwrap();
                    let current_struct_fields = structs.get(&CURRENT_STRUCT_NAME.lock().unwrap().clone())
                        .unwrap_or_else(|| panic!("The struct \"{}\" has not been registered with `#[loadable]`", CURRENT_STRUCT_NAME.lock().unwrap()));

					let mut struct_name: Option<Ident> = None;
					for (other_field_name, field_type) in current_struct_fields {
						if other_field_name == &field_name.to_string() {
							struct_name = Some(Ident::new(field_type, Span::call_site()));
						}
					}

					let Some(struct_name) = struct_name else {
                        panic!("The type of the field \"{}\" has not been registered with `#[loadable]`", field_name);
					};

					let mut current_struct = CURRENT_STRUCT_NAME.lock().unwrap();
					*current_struct = struct_name.to_string();

                    quote! { #struct_name #struct_value }
                }
                FieldValue::Expr(expression) => expression.into_token_stream(),
            };
            quote! { #field_name: #field_tokens }
        });

        tokens.extend(quote! {
            {
                #(#fields_tokens),*
            }
        });
    }
}

struct Wrapper {
    struct_name: Ident,
    value: StructValue,
}

impl Parse for Wrapper {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_name = input.parse::<Ident>()?;
        let mut current_name = CURRENT_STRUCT_NAME.lock().unwrap();
        *current_name = struct_name.to_string();
        let value = StructValue::parse(input)?;
        Ok(Self { struct_name, value })
    }
}

impl ToTokens for Wrapper {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = &self.struct_name;
        let value = &self.value;
        tokens.extend(quote! {
            #struct_name #value
        });
    }
}

#[proc_macro]
pub fn load(input: TokenStream) -> TokenStream {
    let wrapper = parse_macro_input!(input as Wrapper);
    quote! {
        #wrapper
    }
    .into()
}

#[proc_macro_attribute]
pub fn loadable(_attribute: TokenStream, input: TokenStream) -> TokenStream {
    cache_struct(syn::parse::<ItemStruct>(input.clone()).unwrap());
    input
}

use once_cell::sync::Lazy;

static STRUCTS: Lazy<Mutex<HashMap<String, Vec<(String, String)>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CURRENT_STRUCT_NAME: Mutex<String> = Mutex::new(String::new());

/// Store a trait definition for future reference.
fn cache_struct(item: syn::ItemStruct) {
    STRUCTS.lock().unwrap().insert(
        item.ident.to_string(),
        item.fields
            .iter()
            .filter_map(|field| {
                let Type::Path(field_type) = &field.ty else {
                    return None;
                };

                Some((
                    field.ident.as_ref().unwrap().to_string(),
                    field_type.path.segments.last().unwrap().ident.to_string(),
                ))
            })
            .collect(),
    );
}
