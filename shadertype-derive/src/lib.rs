use quote::{quote, quote_spanned};
use syn::{
    parse, parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, ItemStruct, LitInt,
};

#[proc_macro_derive(ShaderType)] // attributes(bind, overlap_mode))]
pub fn derive_shader_type(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(ts as DeriveInput);
    let ident = ast.ident.clone();
    let name = ident.to_string();
    quote_spanned! {ast.span()=>
        impl ::rust_game_engine::renderer::shader_type::ShaderType for #ident {
            const NAME: &'static str = #name;
        }
    }
    .into()
}

#[proc_macro_derive(VertexDataType)] // attributes(bind, overlap_mode))]
pub fn derive_vertex_data_type(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(ts as DeriveInput);
    let Data::Struct(data) = &input.data else {
        return Error::new_spanned(input, "VertexDataType can only be derived on a struct.")
            .into_compile_error()
            .into();
    };
    let struct_ident = input.ident.clone();
    let maude = quote!(::rust_game_engine::renderer::shader_type);
    let vdt = quote!(#maude::VertexDataType);
    let mut n = quote!(0);
    let attributes = data
        .fields
        .iter()
        .map(|f| {
            let name = f.ident.clone().unwrap().to_string();
            let ty = &f.ty;
            n = quote_spanned!(ty.span()=> #n + <#ty as #vdt>::N);
            quote_spanned!(ty.span()=> <#ty as #vdt>::wgsl_fields(#name))
        })
        .collect::<Vec<_>>();

    quote_spanned! {input.span()=>
        impl #vdt for #struct_ident {
            const N: usize = #n;
            fn wgsl_fields(field_name: &'static str) -> Vec<String> {
                [#(#attributes),*].concat()
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}_{}: {}", field_name, i+1, t)).collect()
            }
        }
    }
    .into()
}

#[proc_macro_derive(VertexInput, attributes(location))]
pub fn derive_vertex_input_type(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(ts as DeriveInput);
    let Data::Struct(data) = &input.data else {
        return Error::new_spanned(input, "VertexInput can only be derived on a struct.")
            .into_compile_error()
            .into();
    };

    let struct_ident = input.ident.clone();
    let maude = quote!(::rust_game_engine::renderer::shader_type);
    let mut subsequent_loc = quote!(0);
    let mut errors = vec![];
    let fields = data.fields.iter().map(|f| {
        let loc = 'loc: {
            for attr in &f.attrs {
                if attr.path().is_ident("location") {
                    let Ok(n) = attr.parse_args::<LitInt>() else {
                        errors.push(
                            Error::new_spanned(
                                attr,
                                "argument to location(...) must be an integer literal",
                            )
                            .into_compile_error(),
                        );
                        break;
                    };
                    break 'loc quote!(#n);
                }
            }
            subsequent_loc.clone()
        };
        let name = f.ident.as_ref().unwrap().to_string();
        let checked = quote_spanned! {loc.span()=>
            {
                const _: () = ::const_panic::concat_assert!{
                    #loc >= (#subsequent_loc),
                    "location(", #loc as usize, ") of field ", #name , " is not greater than location value of previous field (", (#subsequent_loc as usize)-1, ")"
                };
                #loc
            }
        };
        let ty = &f.ty;
        subsequent_loc = quote_spanned!(f.span()=> #loc + <#ty as #maude::VertexDataType>::N);
        quote_spanned!(ty.span()=> (#checked, <#ty as #maude::VertexDataType>::wgsl_fields(#name)))
    }).collect::<Vec<_>>();
    if errors.len() > 0 {
        return quote! {
            #(#errors)*
        }
        .into();
    }
    let string_name = struct_ident.to_string();

    quote_spanned! {input.span()=>
        impl #maude::VertexInput for #struct_ident {
            fn definition() -> String {
                let struct_fields = [
                    #(#fields,)*
                ].map(|(loc, f)| {
                    f
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            format!("  @location({}) {},\n", loc+i, s)
                        })
                        .collect::<String>()
                }).concat();
                format!("struct {} {{\n{}}}", #string_name, struct_fields)
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn shader_uniform_type(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = parse_macro_input!(args as parse::Nothing);
    let mut input = syn::parse_macro_input!(input as ItemStruct);
    let struct_ident = input.ident.clone();
    let maude = quote!(::rust_game_engine::renderer::shader_type);
    let (asserts, sb): (Vec<_>, Vec<_>) = input.fields.iter_mut().filter_map(|f| {
        for (i, attr) in  f.attrs.iter_mut().enumerate() {
            if attr.path().is_ident("skip") {
                f.attrs.remove(i);
                return None;
            }
        }
        let id = f
            .ident
            .as_ref()
            .expect("Field did not have a name");
        let name = id.to_string();
        let ty = &f.ty;
        Some((
            quote_spanned! {id.span()=>
                const _: () = {
                    const OFFSET: usize = ::std::mem::offset_of!(#struct_ident, #id);
                    const EXPECTED_ALIGN: usize = <#ty as #maude::ShaderTypeAligned>::ALIGNMENT;
                    const DIFF: usize = OFFSET % EXPECTED_ALIGN;
                    ::const_panic::concat_assert!{
                        DIFF == 0,
                        "Field ", #name, " had offset ", OFFSET, " not matching expected alignment for ", <#ty as #maude::ShaderType>::NAME, " (", EXPECTED_ALIGN, ")."
                    }
                }
            },
            quote_spanned! {ty.span()=>
                format!("  {}: {};", #name, <#ty as #maude::ShaderType>::NAME)
            }
        ))
    }).unzip();

    let name = struct_ident.to_string();
    quote_spanned! {input.span()=>
        #[repr(C, align(16))]
        #[derive(Copy, Clone, ::shadertype_derive::ShaderType, ::bytemuck::Zeroable, ::bytemuck::Pod)]
        #input

        #(#asserts;)*

        impl #maude::ShaderUniformType for #struct_ident {
            fn definition() -> String {
                format!("struct {} {{\n{}\n}}\n", #name, [#(#sb,)*].join("\n"))
            }
        }
    }
    .into()
}
