use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Error, Expr, ExprLit, Fields, Lit,
    Result, Token,
};

#[proc_macro_derive(ControlSet, attributes(bind, overlap_mode))]
pub fn derive_control_set(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(ts as DeriveInput);
    expand_control_set(ast)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_control_set(ast: DeriveInput) -> Result<TokenStream> {
    let control_set_data = ControlSetData::try_from(ast)?;
    let control_enum = control_set_data.control_enum();
    let controlset_impl = control_set_data.controlset_impl()?;

    let result = quote!(
        #control_enum

        #controlset_impl
    )
    .into();
    Ok(result)
}

struct ControlSetData {
    input: DeriveInput,
    enum_ident: Ident,
    struct_fields: Fields,
    enum_fields: Vec<Ident>,
}

impl std::convert::TryFrom<DeriveInput> for ControlSetData {
    type Error = syn::Error;

    fn try_from(input: DeriveInput) -> std::result::Result<Self, Self::Error> {
        let enum_ident = format_ident!("{}DeriveControls", input.ident);
        let Data::Struct(data) = input.clone().data else {
            return Err(Error::new_spanned(
                input,
                "ControlSet can only be derived on a struct.",
            ));
        };
        let struct_fields = data.fields.clone();
        let enum_fields = struct_fields
            .iter()
            .filter_map(|field| field.ident.as_ref().map(titlecase_ident))
            .collect::<Vec<_>>();
        Ok(Self {
            input,
            enum_ident,
            struct_fields,
            enum_fields,
        })
    }
}

impl ControlSetData {
    fn control_enum(&self) -> TokenStream {
        let ident = self.enum_ident.clone();
        let enum_fields = self.enum_fields.clone();
        quote!(
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
            enum #ident {
                #(#enum_fields,)*
            }
        )
    }

    fn init_fields(&self) -> Result<TokenStream> {
        let mut fields = vec![];
        for f in self.struct_fields.iter() {
            let field_ident = f.ident.clone().expect("field must be named");
            let ty = f.ty.clone();
            let mut bindings = vec![];
            let mut overlap_mode: Option<TokenStream> = None;
            for a in &f.attrs {
                if a.path().is_ident("bind") {
                    let e = a.meta.require_list()?;
                    let args =
                        e.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                    for item in args {
                        bindings.push(quote_spanned! (item.span()=>
                            (#item).into()
                        ));
                    }
                    if bindings.len() == 0 {
                        return Err(Error::new_spanned(e, "bind attribute expects arguments"));
                    }
                } else if a.path().is_ident("overlap_mode") {
                    let kv = a.meta.require_name_value()?;
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &kv.value
                    {
                        overlap_mode = Some(match s.value().as_str() {
                            "first" => quote!(::rust_game_engine::input::OverlapMode::First),
                            "latest" => quote!(::rust_game_engine::input::OverlapMode::Latest),
                            "neutral" => quote!(::rust_game_engine::input::OverlapMode::Neutral),
                            x => {
                                return Err(Error::new_spanned(
                                    kv.value.clone(),
                                    format!("Unrecognized overlap_mode '{}'", x),
                                ));
                            }
                        });
                    } else {
                        return Err(Error::new_spanned(
                            kv.value.clone(),
                            format!("Expected overlap_mode as string literal"),
                        ));
                    }
                }
            }
            let with_overlap_mode = overlap_mode.map(|e| {
                quote_spanned!(e.span()=>
                    . with_overlap_mode ( #e )
                )
            });
            fields.push(quote_spanned!(f.span()=>
                #field_ident : <#ty>::new(vec![
                    #(#bindings,)*
                ])
                #with_overlap_mode
            ));
        }

        Ok(quote!(#(#fields,)*))
    }

    fn controls_mapping(&self, f: impl Fn(TokenStream, Ident) -> TokenStream) -> TokenStream {
        let enum_ident = self.enum_ident.clone();
        let struct_fields = self.struct_fields.iter().map(|s| s.ident.clone().unwrap());
        let mappings = self
            .enum_fields
            .iter()
            .map(|ident| {
                let enum_ident = enum_ident.clone();
                quote!(#enum_ident :: #ident)
            })
            .zip(struct_fields)
            .map(|(e, s)| f(e, s));
        quote!(
            match control {
                #(#mappings,)*
            }
        )
    }

    fn controlset_impl(&self) -> Result<TokenStream> {
        let struct_ident = self.input.ident.clone();
        let enum_ident = self.enum_ident.clone();
        let enum_fields = self.enum_fields.iter().map(|ident| {
            let enum_ident = enum_ident.clone();
            quote!(#enum_ident :: #ident)
        });
        let field_update_mapping =
            self.controls_mapping(|e, s| quote!(#e => self.#s.update(change)));
        let field_bound_inputs_mapping =
            self.controls_mapping(|e, s| quote!(#e => self.#s.bound_inputs()));
        let field_changed_mapping = self.controls_mapping(|e, s| quote!(#e => self.#s.changed()));
        let clear_field_changed_mapping =
            self.controls_mapping(|e, s| quote!(#e => self.#s.clear_changed()));
        let field_state_mapping = self.controls_mapping(|e, s| quote!(#e => self.#s.state()));
        let init_fields = self.init_fields()?;

        Ok(quote!(
            impl Default for #struct_ident {
                fn default() -> Self {
                    Self{
                        #init_fields
                    }
                }
            }
            impl ::rust_game_engine::input::ControlSet for #struct_ident {
                type Control = #enum_ident;

                fn controls() -> &'static [Self::Control] {
                    &[
                        #(#enum_fields,)*
                    ]
                }

                fn handle_input(&mut self, control: &Self::Control, change: Option<::rust_game_engine::input::InputChange>) {
                    #field_update_mapping
                }

                fn bound_inputs(&self, control: &Self::Control) -> Vec<::rust_game_engine::input::AnyInput> {
                    #field_bound_inputs_mapping
                }

                fn control_changed(&self, control: &Self::Control) -> bool {
                    #field_changed_mapping
                }

                fn clear_control_changed(&mut self, control: &Self::Control) {
                    #clear_field_changed_mapping
                }

                fn control_state(&self, control: &Self::Control) -> ::rust_game_engine::input::InputState {
                    #field_state_mapping
                }
            }
        ))
    }
}

fn titlecase_ident(ident: &Ident) -> Ident {
    let s = ident.to_string();
    let mut chars = s.chars();
    let name = chars
        .next()
        .unwrap()
        .to_uppercase()
        .chain(chars)
        .collect::<String>();
    Ident::new(&name, Span::call_site())
}
