use quote::{quote, quote_spanned, ToTokens};
use syn::{
    Data, DeriveInput, Error, Ident, ItemStruct, LitInt, Path, Token, parse::{self, Parse}, parse_macro_input, spanned::Spanned
};

#[proc_macro_derive(ShaderNamedType)] // attributes(bind, overlap_mode))]
pub fn derive_shader_named_type(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(ts as DeriveInput);
    let ident = ast.ident.clone();
    let name = ident.to_string();
    quote_spanned! {ast.span()=>
        impl ::rust_game_engine::renderer::shader_type::ShaderNamedType for #ident {
            fn wgsl_name() -> String {
                #name.into()
            }
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

#[proc_macro_derive(VertexInput, attributes(step_mode, location))]
pub fn derive_vertex_input_type(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(ts as DeriveInput);
    let Data::Struct(data) = &input.data else {
        return Error::new_spanned(input, "VertexInput can only be derived on a struct.")
            .into_compile_error()
            .into();
    };
    let mut errors = vec![];
    let mut step_mode = quote!(::wgpu::VertexStepMode::Vertex);
    for attr in &input.attrs {
        if attr.path().is_ident("step_mode") {
            match attr.parse_args::<Ident>() {
                Ok(id) if id == "vertex" => {}
                Ok(id) if id == "instance"=> step_mode = quote!(::wgpu::VertexStepMode::Instance),
                _ => {
                        errors.push(
                            Error::new_spanned(
                                attr,
                                "argument to step_mode(...) must be either vertex or instance",
                            )
                            .into_compile_error(),
                        );
                        break;
                }
            }
        }
    }
    if errors.len() > 0 {
        return quote! {
            #(#errors)*
        }
        .into();
    }

    let struct_ident = input.ident.clone();
    let maude = quote!(::rust_game_engine::renderer::shader_type);
    let mut subsequent_loc = quote!(0);
    let mut attrs = vec![];
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
        attrs.push(quote_spanned!{ty.span()=> 
            (#loc, <#ty as #maude::VertexDataType>::vertex_formats())
        });
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
        unsafe impl ::bytemuck::Zeroable for #struct_ident { }
        unsafe impl ::bytemuck::Pod for #struct_ident { }
        impl #maude::VertexInput for #struct_ident {
            fn definition(location_offset: u32) -> String {
                #maude::define_wgsl_struct(#string_name, [
                    #(#fields,)*
                ], location_offset)
            }

            fn vertex_attributes() -> Vec<::wgpu::VertexAttribute> {
                #maude::expand_vertex_attributes([
                    #(#attrs,)*
                ])
            }

            fn step_mode() -> ::wgpu::VertexStepMode {
                #step_mode
            }
        }
    }
    .into()
}

fn is_basic_wgsl_type(t: &str) -> bool {
    let vec_re = regex::Regex::new(r"^vec[2-4]([ifuh]|<(.*)>)$").unwrap();
    let mat_re = regex::Regex::new(r"^mat[2-4]x[2-4]([fh]|<(.*)>)$").unwrap();
    let array_re = regex::Regex::new(r"^array<([^,]*)(,\s*\d+)?>$").unwrap();

    match t {
        "bool" | "u32" | "i32" | "f32" | "f16" => true,
        "atomic<u32>" | "atomic<i32>" => true,
        "texture_external" | "texture_depth_2d" | "texture_depth_2d_array" | "texture_depth_cube" | "texture_depth_cube_array" => true,
        "sampler" | "sampler_comparison" => true,
        _ if t.starts_with("texture_1d<") => true,
        _ if t.starts_with("texture_storage_1d<") => true,
        _ if t.starts_with("texture_2d<") => true,
        _ if t.starts_with("texture_storage_2d<") => true,
        _ if t.starts_with("texture_2d_array<") => true,
        _ if t.starts_with("texture_storage_2d_array<") => true,
        _ if t.starts_with("texture_3d<") => true,
        _ if t.starts_with("texture_storage_3d<") => true,
        _ if t.starts_with("texture_multisampled_2d<") => true,
        _ if t.starts_with("texture_depth_multisampled_2d<") => true,
        _ => {
            if vec_re.is_match(t) {
                return true;
            }
            if mat_re.is_match(t) {
                return true;
            }
            if let Some(m) = array_re.captures(t) {
                return is_basic_wgsl_type(m.get(1).unwrap().as_str());
            }
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::is_basic_wgsl_type;

    #[test]
    fn test_is_basic_wgsl_type() {
        assert!(is_basic_wgsl_type("i32"));
        assert!(is_basic_wgsl_type("vec2<i32>"));
        assert!(is_basic_wgsl_type("array<i32>"));
        assert!(is_basic_wgsl_type("array<i32, 3>"));
        assert!(!is_basic_wgsl_type("Blah"));
        assert!(!is_basic_wgsl_type("array<Blah>"));
        assert!(!is_basic_wgsl_type("array<Blah, 12>"));
    }
}

struct ShaderUniformDataArgs {
    _raw_token: Ident,
    _eq_token: Token![=],
    raw_type: Path,
}

impl Parse for ShaderUniformDataArgs {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let _raw_token: Ident = input.parse()?;
        if _raw_token != "Raw" {
            return Err(syn::Error::new_spanned(_raw_token, "expected `Raw`"));
        }
        Ok(Self{
            _raw_token,
            _eq_token: input.parse()?,
            raw_type: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn shader_uniform_data(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut repr_attr = quote!();
    let mut raw_type = quote!(Self);
    let mut self_is_raw = true;
    if args.is_empty() {
        let _ = parse_macro_input!(args as parse::Nothing);
        repr_attr = quote!(#[repr(C, align(16))]);
    } else {
        let args = parse_macro_input!(args as ShaderUniformDataArgs);
        raw_type = args.raw_type.to_token_stream();
        self_is_raw = false;
    }
    let mut input = syn::parse_macro_input!(input as ItemStruct);
    let struct_ident = input.ident.clone();
    let name = struct_ident.to_string();
    let maude = quote!(::rust_game_engine::renderer::shader_type);

    if !self_is_raw {
    return quote_spanned! {input.span()=>
        #repr_attr
        #input

        impl #maude::ShaderUniformData for #struct_ident {
            type Raw = #raw_type;
            fn definition() -> String {
                <#raw_type as #maude::ShaderUniformData>::definition()
            }
            fn raw(&self) -> Self::Raw {
                From::from(self.clone())
            }
        }
        impl #maude::ShaderNamedType for #struct_ident {
            fn wgsl_name() -> String {
                <#raw_type as #maude::ShaderNamedType>::wgsl_name()
            }
        }
        impl #maude::ShaderDefinedType for #struct_ident {
            fn definitions() -> Vec<String> {
                <#raw_type as #maude::ShaderDefinedType>::definitions()
            }
        }
    }
    .into();
    }

    let mut aligns = vec![];
    let mut defs = vec![];
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

        aligns.push(quote_spanned!(ty.span()=> <#ty as #maude::ShaderTypeAligned>::ALIGNMENT));
        
        defs.push(quote_spanned!(ty.span()=> <#ty as #maude::ShaderDefinedType>::definitions()));
        Some((
            quote_spanned! {id.span()=>
                const _: () = {
                    const OFFSET: usize = ::std::mem::offset_of!(#struct_ident, #id);
                    const EXPECTED_ALIGN: usize = <#ty as #maude::ShaderTypeAligned>::ALIGNMENT;
                    const DIFF: usize = OFFSET % EXPECTED_ALIGN;
                    ::const_panic::concat_assert!{
                        DIFF == 0,
                        "Field ", #name, " had offset ", OFFSET, " not matching expected alignment for ", stringify!(#ty), " (", EXPECTED_ALIGN, ")." }
                }
            },
            quote_spanned! {ty.span()=>
                format!("  {}: {},", #name, <#ty as #maude::ShaderNamedType>::wgsl_name())
            }
        ))
    }).unzip();

    // TODO: need to emit #raw_type::definition() if !self_is_raw, same with SDT
    quote_spanned! {input.span()=>
        // unsafe impl ::bytemuck::Zeroable for #struct_ident { }
        // unsafe impl ::bytemuck::Pod for #struct_ident { }

        // #[repr(C, align(16))]
        // #[derive(Copy, Clone)]
        #repr_attr
        #input

        #(#asserts;)*

        impl #maude::ShaderTypeAligned for #struct_ident {
            const ALIGNMENT: usize = {
                let aligns = &[
                    #(#aligns,)*
                ];
                let mut i = 0;
                let mut max = 0;
                while i < aligns.len() {
                    if aligns[i] > max {
                        max = aligns[i];
                    }
                    i += 1;
                }
                max
            };
        }

        impl #maude::ShaderUniformData for #struct_ident {
            type Raw = #raw_type;
            fn definition() -> String {
                format!("struct {} {{\n{}\n}}\n", #name, [#(#sb,)*].join("\n"))
            }
            fn raw(&self) -> Self::Raw {
                From::from(self.clone())
            }
        }
        impl #maude::ShaderNamedType for #struct_ident {
            fn wgsl_name() -> String {
                #name.into()
            }
        }
        impl #maude::ShaderDefinedType for #struct_ident {
            fn definitions() -> Vec<String> {
                [
                    #(#defs,)*
                    vec![<Self as #maude::ShaderUniformData>::definition()],
                ].into_iter().flatten().collect()
            }
        }
    }
    .into()
}
