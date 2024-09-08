use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Meta, NestedMeta};

#[proc_macro_derive(ToolCodegen, attributes(tool))]
pub fn tool_codegen_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the enum
    let name = input.ident;

    // Ensure the input is an enum
    let data_enum = match input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("ToolCodegen can only be used on enums"),
    };

    // Prepare to collect tool information
    let mut tool_info = Vec::new();

    // Iterate over each variant of the enum
    for variant in data_enum.variants {
        let variant_name = variant.ident;
        let mut tool_name = String::new();
        let mut tool_description = String::new();

        // Find the `#[tool(...)]` attribute for each variant
        for attr in variant.attrs.iter().filter(|a| a.path().is_ident("tool")) {
            // if let Ok(meta) = attr.parse_nested_meta() {
            //     if let Meta::List(meta_list) = meta {
            //         for nested_meta in meta_list.nested {
            //             match nested_meta {
            //                 NestedMeta::Meta(Meta::NameValue(name_value)) if name_value.path.is_ident("name") => {
            //                     if let syn::Lit::Str(lit_str) = name_value.lit {
            //                         tool_name = lit_str.value();
            //                     }
            //                 }
            //                 NestedMeta::Meta(Meta::NameValue(name_value)) if name_value.path.is_ident("description") => {
            //                     if let syn::Lit::Str(lit_str) = name_value.lit {
            //                         tool_description = lit_str.value();
            //                     }
            //                 }
            //                 _ => {}
            //             }
            //         }
            //     }
            // }
        }

        // Collect the extracted information
        tool_info.push(quote! {
            {
                "name": #tool_name,
                "description": #tool_description,
                "variant": stringify!(#variant_name)
            }
        });
    }

    // Generate the output Rust code that returns the JSON-like descriptions
    let expanded = quote! {
        impl #name {
            pub fn describe_tools() -> Vec<serde_json::Value> {
                vec![
                    #(#tool_info),*
                ]
            }
        }
    };

    // Convert the generated code into a token stream and return it
    TokenStream::from(expanded)
}
