use {
    proc_macro::TokenStream as OriginalTokenStream,
    proc_macro2::{TokenStream, TokenTree},
    quote::quote,
    std::collections::HashSet,
    syn::{
        parse::{Parse, ParseStream},
        parse_macro_input,
        punctuated::Punctuated,
        DeriveInput,
        Ident,
        Token,
    },
};

struct Args {
    vars: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        // parses a,b,c, or a,b,c where a,b and c are Indent
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn modx(attr: OriginalTokenStream, item: OriginalTokenStream) -> OriginalTokenStream {
    let clone_item = item.clone();
    let input = parse_macro_input!(clone_item as DeriveInput);
    let args = parse_macro_input!(attr as Args);

    // We'll collect the identifiers from the attributes
    let derive_from: Vec<String> = args.vars.iter().map(|e| e.to_string()).collect();

    let struct_name = &input.ident;

    // Get the fields of the struct
    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &input.data {
        fields
    } else {
        return TokenStream::from(quote! {
            compile_error!("Only structs are supported for this macro");
        })
        .into();
    };

    // Data that will be used later on
    let mut signal_fields = fields.clone();
    let mut all_idents_types = vec![];

    for field in &mut signal_fields {
        if let Some(ident) = &field.ident {
            // Push the new ident
            all_idents_types.push((ident.clone(), field.ty.clone()));

            // Type of the current field
            let field_type = field.ty.clone();

            // Create the signal
            let signal_type = quote! { Signal<#field_type> }.into();
            field.ty = parse_macro_input!(signal_type as syn::Type);
        }
    }

    // Remove {} around fields
    let param_fields = remove_delimiter_and_pass(quote! { #fields });

    // Convert default value to signal
    let signal_idents = all_idents_types.iter().map(|(ident, _)| {
        quote! {
            #ident: use_signal(|| #ident),
        }
    });

    // Implement the automatic clone
    let impl_signal_idents = all_idents_types.iter().map(|(ident, ty)| {
        quote! {
            impl #struct_name {
                pub fn #ident(&self) -> #ty {
                    self.#ident.read().clone()
                }
            }
        }
    });

    // Implement default values if there is "default"
    let impl_default = if derive_from.contains(&"Default".to_string()) {
        let default_values = all_idents_types.iter().map(|(ident, ty)| {
            let ty_corrected = quote!(#ty).to_string().replace("<", "::<");
            let parsed_type: syn::Type = syn::parse_str(&ty_corrected).unwrap();

            quote! {
                #ident: use_signal(|| #parsed_type::default()),
            }
        });

        quote! {
            impl Default for #struct_name {
                fn default() -> #struct_name {
                    #struct_name {
                        #(#default_values)*
                    }
                }
            }
        }
    } else {
        quote!()
    };

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq)]
        struct #struct_name
            #signal_fields

        impl #struct_name {
            pub fn new_signal(#param_fields) -> #struct_name {
                #struct_name {
                    #(#signal_idents)*
                }
            }
        }

        #(#impl_signal_idents)*

        #impl_default
    }
    .into()
}

fn remove_delimiter_and_pass(stream: TokenStream) -> Option<TokenStream> {
    // Iterate through the token stream
    for token in stream {
        // Check if the token is a Group
        if let TokenTree::Group(group) = token {
            // Extract the inner stream from the Group
            let inner_stream = group.stream();

            // Pass the inner stream to a function
            return Some(inner_stream);
        }
    }

    None
}
