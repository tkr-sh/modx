use {
    proc_macro::TokenStream as OriginalTokenStream,
    proc_macro2::TokenStream,
    quote::quote,
    std::{collections::HashSet, str::FromStr},
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

#[derive(PartialEq, Eq)]
enum TypeOfField {
    Signal,
    Resource,
    // ServerFuture,
    Props,
}

/// Declare a struct as a modx store.
///
/// ## Usage
/// ```
/// #[modx::store]
/// struct MyStore {
///     name: String,
///     age: u8,
/// }
/// ```
///
/// ## Attributes
/// By default, this struct will have some implementation. The main one being `new`.
/// New will ONLY work if all fields have a type that implements the `Default` trait.
/// If one does not, you will need to use the `#[modx::props]` macro to take some parameters.
///
/// ## Example
///
/// When every fields implements default.
/// ```
/// #[modx::store]
/// struct MyStore {
///     name: String,
///     age: u8,
/// }
///
/// let store = MyStore::new();  // { name: String(""), age: 0 }
/// ```
///
/// When the component takes props
/// ```
/// #[modx::props(age)]
/// #[modx::store]
/// struct MyStore {
///     name: String,
///     age: u8,
/// }
///
/// let store = MyStore::new(MyStoreProps { age: 21 });  // { name: String(""), age: 21 }
/// ```
///
///
/// ## With other macros
/// By default, every field is a Signal.
/// But this can be a bit concerning if you want to use a `resource` for example.
/// If you want to use a resource for example, you can use the `modx::resource` procedural macro.
///
/// ```
/// #[modx::resource(age)]
/// #[modx::store]
/// struct MyStruct {
///     name: String, // <- Will be a Signal<String>
///     age: u8,      // <- Will be a Resource<u8>
/// }
/// ```
#[proc_macro_attribute]
pub fn store(_: OriginalTokenStream, item: OriginalTokenStream) -> OriginalTokenStream {
    let clone_item = item.clone();
    let input = parse_macro_input!(clone_item as DeriveInput);

    let struct_name = &input.ident;
    let struct_visibility = &input.vis;

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
    let mut modified_fields = fields.clone();
    let mut all_idents_types = vec![];
    let mut props_idents = vec![];

    for field in &mut modified_fields {
        if let Some(ident) = &field.ident {
            if !ident.to_string().starts_with("_modx_reserved") {
                // Push the new ident
                all_idents_types.push((ident.clone(), field.ty.clone(), TypeOfField::Signal));

                // Type of the current field
                let field_type = field.ty.clone();

                // Create the signal
                let signal_type = quote! { Signal<#field_type> }.into();
                field.ty = parse_macro_input!(signal_type as syn::Type);
            } else if ident.to_string().starts_with("_modx_reserved_resource_") {
                // Push the new ident
                let new_name = ident.to_string().replace("_modx_reserved_resource_", "");
                let new_ident = Ident::new(&new_name, proc_macro2::Span::call_site());
                field.ident = Some(new_ident.clone());
                all_idents_types.push((new_ident, field.ty.clone(), TypeOfField::Resource));

                let field_type = field.ty.clone();
                let signal_type = quote! { Resource<#field_type> }.into();
                field.ty = parse_macro_input!(signal_type as syn::Type);
            } else if ident.to_string().starts_with("_modx_reserved_props_") {
                // Push the new ident
                let new_name = ident.to_string().replace("_modx_reserved_props_", "");
                let new_ident = Ident::new(&new_name, proc_macro2::Span::call_site());
                field.ident = Some(new_ident.clone());
                all_idents_types.push((new_ident.clone(), field.ty.clone(), TypeOfField::Props));

                let field_type = field.ty.clone();
                let signal_type = quote! { Signal<#field_type> }.into();
                field.ty = parse_macro_input!(signal_type as syn::Type);
                props_idents.push((new_ident.clone(), field_type));
            }
        }
    }

    // Implement the automatic clone
    let impl_signal_idents = all_idents_types.iter().map(|(ident, ty, type_of_field)| {
        match type_of_field {
            TypeOfField::Signal | TypeOfField::Props => {
                quote! {
                    impl #struct_name {
                        pub fn #ident(&self) -> #ty {
                            self.#ident.read().clone()
                        }
                    }
                }
            },
            _ => quote! {},
        }
    });

    // Implement default values if there is "default"
    let impl_default = {
        // Convert type to type::default() for every type
        let default_values = all_idents_types.iter().map(|(ident, ty, type_of_field)| {
            let ty_corrected = quote!(#ty).to_string().replace("<", "::<");
            let parsed_type: syn::Type = match syn::parse_str(&ty_corrected) {
                Ok(t) => t,
                Err(why) => {
                    return why.to_compile_error().into();
                },
            };

            match type_of_field {
                TypeOfField::Signal => quote! {
                    #ident: use_signal(|| #parsed_type::default()),
                },
                TypeOfField::Props => quote! {
                    #ident: use_signal(|| props.#ident),
                },
                // TODO: Cange case
                _ => quote! { #ident: use_resource(move || async move { unsafe { std::mem::zeroed() } }), },
            }
        });

        // The resources that we need to assign just after creation.
        // We NEED to do that because for now, its uninialized with `unsafe { std::mem::zeroed() }`
        let alter_resources = all_idents_types.iter().map(|(ident, _, type_of_field)| {
            if *type_of_field == TypeOfField::Resource {
                quote! { default_struct.#ident = use_resource(move || async move { default_struct.#ident().await } ); }
            } else {
                quote!{}
            }
        });

        // If there is no field that should be used as a props, we just return the default struct
        // that takes no parameter.
        if props_idents.len() == 0 {
            // Implement default
            quote! {
                impl #struct_name {
                    pub fn new() -> #struct_name {
                        let mut default_struct = #struct_name {
                            #(#default_values)*
                        };

                        #(#alter_resources)*

                        default_struct
                    }
                }
            }
        }
        // Else, we create a struct #(#struct_name)Props that takes the props
        else {
            let struct_props_fields = props_idents
                .iter()
                .map(|(ident, ty)| quote! ( #ident: #ty, ))
                .collect::<Vec<_>>();

            let structprops_name = quote!(#struct_name).to_string();
            let structprops_name: syn::Type =
                match syn::parse_str(&format!("{structprops_name}Props")) {
                    Ok(t) => t,
                    Err(why) => {
                        return why.to_compile_error().into();
                    },
                };

            // Implement default
            quote! {
                #[derive(Debug)]
                #struct_visibility struct #structprops_name {
                    #(#struct_props_fields)*
                }
                impl #struct_name {
                    pub fn new(props: #structprops_name) -> #struct_name {
                        let mut default_struct = #struct_name {
                            #(#default_values)*
                        };

                        #(#alter_resources)*

                        default_struct
                    }
                }
            }
        }
    };

    quote! {
        #[derive(Copy, Clone)]
        #struct_visibility struct #struct_name
            #modified_fields

        #(#impl_signal_idents)*

        #impl_default
    }
    .into()
}

/// Get resources with a function
///
/// ## Usage
/// ```
/// #[modx::resource(fetch_cat_url)]
/// #[modx::store]
/// struct MyStore {
///     number_of_cats: usize,
///     fetch_cat_url: Result<String>,
/// }
///
/// impl MyStore {
///     async fn fetch_cat_url() -> Result<String> {
///         self.number_of_cats += 1;
///
///         reqwest::get("http://localhost/cat")
///             .await
///             .unwrap()
///             .json::<ApiResponse>()
///             .await
///     }
/// }
///
/// let store = MyStore::new();
/// match &*store.fetch_cat_url.read()  {
///     Some(Ok(url)) =>
///         rsx! {
///             div {
///                 img {
///                     max_width: "500px",
///                     max_height: "500px",
///                     src: url
///                 }
///             }
///             "Cat N°{store.number_of_cats}"
///         },
///     Some(Err(_)) => rsx! { "An error occured while getting a cat :(" },
///     None => rsx!( "No cat for now." ),
/// }
/// ```
///
/// ## Attributes
/// - Every attributes passed in the `resource` procedural macro needs to be implemented as a function
/// in this particular struct and also being a field of this struct with the proper type.
///
/// - Functions that are concerned by this macro need to be async and shouldn't take any parameter.
#[proc_macro_attribute]
pub fn resource(attr: OriginalTokenStream, item: OriginalTokenStream) -> OriginalTokenStream {
    let clone_item = item.clone();
    let input = parse_macro_input!(clone_item as DeriveInput);
    let args = parse_macro_input!(attr as Args);

    // We'll collect the identifiers from the attributes
    let resource_fields: Vec<String> = args.vars.iter().map(|e| e.to_string()).collect();

    let struct_name = &input.ident;
    let struct_visibility = &input.vis;

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
    let mut renamed_fields = fields.clone();

    for field in &mut renamed_fields {
        if let Some(ident) = &mut field.ident {
            let string_ident = ident.to_string();
            if resource_fields.contains(&string_ident) {
                let new_name = format!("_modx_reserved_resource_{}", ident.to_string());
                field.ident = Some(Ident::new(&new_name, proc_macro2::Span::call_site()));
            }
        }
    }

    let data = item
        .clone()
        .into_iter()
        .collect::<Vec<proc_macro::TokenTree>>();
    let mut proc_macro_attributes = vec![];
    let mut i = 0;
    while let Some(proc_macro::TokenTree::Punct(punct)) = data.get(i) {
        if punct.as_char() == '#' {
            if let Some(proc_macro::TokenTree::Group(group)) = data.get(i + 1) {
                proc_macro_attributes.push(format!("#{group}"));
                i += 2;
            }
        }
    }

    let attributes_string = proc_macro_attributes
        .iter()
        .map(|proc_macro_attribute| {
            match OriginalTokenStream::from_str(&proc_macro_attribute.clone()) {
                Ok(v) => v.into(),
                // Fix with a better error
                Err(_why) => {
                    return TokenStream::from(quote! {
                        compile_error!("A bad proc_macro_attr was found: {proc_macro_attribute}");
                    })
                    .into();
                },
            }
        })
        .collect::<Vec<TokenStream>>();

    let data = quote! {
        #(#attributes_string)*
        #struct_visibility struct #struct_name
            #renamed_fields
    };

    data.into()
}

/// Add some props to a modx store
///
/// ## Usage
/// ```
/// // Adds price and name a props o the component
/// #[modx::props(price, name)]
/// #[modx::store]
/// struct MyStore {
///     price: usize,
///     name: String,
///     sold_today: usize,
/// }
///
/// // Create the store with props
/// let store = MyStore::new(
///     MyStoreProps{
///         price: 10,
///         name: String::from("item")
///     }
/// )
/// ```
///
/// ## Attributes
/// This procedural macro automatically create a struct with the same name as the original struct +
/// Props in suffix, that will have in field, all the props defined in the #[modx:props] macro.
///
/// Every props is still a signal so you can easily modify them, copy them and see the changes.
#[proc_macro_attribute]
pub fn props(attr: OriginalTokenStream, item: OriginalTokenStream) -> OriginalTokenStream {
    let clone_item = item.clone();
    let input = parse_macro_input!(clone_item as DeriveInput);
    let args = parse_macro_input!(attr as Args);

    // We'll collect the identifiers from the attributes
    let resource_fields: Vec<String> = args.vars.iter().map(|e| e.to_string()).collect();

    let struct_name = &input.ident;
    let struct_visibility = &input.vis;

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
    let mut renamed_fields = fields.clone();

    for field in &mut renamed_fields {
        if let Some(ident) = &mut field.ident {
            let string_ident = ident.to_string();
            if resource_fields.contains(&string_ident) {
                let new_name = format!("_modx_reserved_props_{}", ident.to_string());
                field.ident = Some(Ident::new(&new_name, proc_macro2::Span::call_site()));
            }
        }
    }

    let data = item
        .clone()
        .into_iter()
        .collect::<Vec<proc_macro::TokenTree>>();

    let mut proc_macro_attributes = vec![];
    let mut i = 0;
    while let Some(proc_macro::TokenTree::Punct(punct)) = data.get(i) {
        if punct.as_char() == '#' {
            if let Some(proc_macro::TokenTree::Group(group)) = data.get(i + 1) {
                proc_macro_attributes.push(format!("#{group}"));
                i += 2;
            }
        }
    }

    let attributes_string = proc_macro_attributes
        .iter()
        .map(|proc_macro_attribute| {
            match OriginalTokenStream::from_str(&proc_macro_attribute.clone()) {
                Ok(v) => v.into(),
                // Fix with a better error
                Err(_why) => {
                    return TokenStream::from(quote! {
                        compile_error!("A bad proc_macro_attr was found: {proc_macro_attribute}");
                    })
                    .into();
                },
            }
        })
        .collect::<Vec<TokenStream>>();

    let data = quote! {
        #(#attributes_string)*

        #struct_visibility struct #struct_name
            #renamed_fields
    };

    data.into()
}
