use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Ident, Result, Type, TypeArray, Visibility};

macro_rules! bail {
    ($msg:expr) => {
        return ::core::result::Result::Err(::syn::Error::new(
            ::proc_macro2::Span::call_site(),
            $msg,
        ))
    };
    ($span:expr, $msg:expr) => {
        return ::core::result::Result::Err(::syn::Error::new_spanned($span, $msg))
    };
}

#[proc_macro_derive(RegMap, attributes(reg))]
pub fn reg_map_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);

    // Build the trait implementation
    impl_reg(&input).unwrap_or_else(|err| err.into_compile_error().into())
}

fn impl_reg(ast: &DeriveInput) -> Result<TokenStream> {
    let name = &ast.ident;
    let vis = &ast.vis;
    let ptr_vis = parse_visibility(vis)?;

    // check if using a compatible repr
    check_repr(ast)?;

    if let Data::Struct(DataStruct {
        struct_token: _,
        ref fields,
        semi_token: _,
    }) = ast.data
    {
        let ptr_name = Ident::new(&format!("{}Ptr", name), Span::call_site());
        let mod_name = Ident::new(&format!("_mod_{}", name), Span::call_site());
        let mut all_methods = quote!();
        if let Fields::Named(named) = fields {
            for field in named.named.iter() {
                all_methods.extend(parse_field(field)?);
            }
        } else {
            bail!(ast, "RegMap derive supports only structs with named fields");
        }
        let doc_msg_top = format!("A pointer to the register map `{name}`.");
        let doc_msg_from_nonnull = format!(
            "\
            Creates a new `{ptr_name}`, a pointer to `{name}`.\n\
            \n\
            # Safety\n\
            - `ptr` must point to a valid instance of `{name}`;\n\
            - `ptr` must be valid for the whole lifetime `'a`;\n\
            - all fields of `{name}` must allow volatile reads/writes."
        );
        let doc_msg_from_ptr = format!(
            "\
            Creates a new `{ptr_name}`, a pointer to `{name}`.\n\
            \n\
            # Safety\n\
            - `ptr` must not be null;\n\
            - `ptr` must point to a valid instance of `{name}`;\n\
            - `ptr` must be valid for the whole lifetime `'a`;\n\
            - all fields of `{name}` must allow volatile reads/writes."
        );
        let doc_msg_from_mut =
            format!("Return a pointer to `{name}` from a mutable (exclusive) reference.");
        let all = quote!(
            #[allow(non_snake_case)]
            mod #mod_name {
                use super::*;
                #[doc = #doc_msg_top]
                #ptr_vis struct #ptr_name<'a> {
                    ptr: ::core::ptr::NonNull<#name>,
                    _ref: ::core::marker::PhantomData<&'a #name>,
                }
                impl<'a> #ptr_name<'a> {
                    #[doc = #doc_msg_from_nonnull]
                    #[inline]
                    const unsafe fn from_nonnull(ptr: ::core::ptr::NonNull<#name>) -> Self {
                        Self {
                            ptr,
                            _ref: ::core::marker::PhantomData,
                        }
                    }

                    #[doc = #doc_msg_from_ptr]
                    #[inline]
                    pub const unsafe fn from_ptr(ptr: *mut #name) -> Self {
                        Self::from_nonnull(::core::ptr::NonNull::new_unchecked(ptr))
                    }

                    #[doc = #doc_msg_from_mut]
                    #[inline]
                    pub fn from_mut(reg: &'a mut #name) -> Self {
                        // safe because we are the only borrowers (&mut)
                        // and the borrow is valid for 'a
                        unsafe { Self::from_ptr(reg) }
                    }

                    /// Returns a raw pointer to the underlying register map.
                    #[inline]
                    pub const fn as_ptr(&self) -> *mut #name {
                        self.ptr.as_ptr()
                    }
                    #all_methods
                }
                unsafe impl<'a> ::reg_map::RegMapPtr<'a> for #ptr_name<'a> {
                    type RegMap = #name;
                    #[inline]
                    unsafe fn from_nonnull(ptr: ::core::ptr::NonNull<Self::RegMap>) -> Self {
                        Self::from_nonnull(ptr)
                    }
                    #[inline]
                    unsafe fn from_ptr(ptr: *mut Self::RegMap) -> Self {
                        Self::from_ptr(ptr)
                    }
                    #[inline]
                    fn from_mut(reg: &'a mut Self::RegMap) -> Self {
                        Self::from_mut(reg)
                    }
                    #[inline]
                    fn as_ptr(&self) -> *mut Self::RegMap {
                        self.as_ptr()
                    }
                }
            }
            #vis use #mod_name::#ptr_name;
        );
        Ok(all.into())
    } else {
        bail!(ast, "RegMap derive supports only structs")
    }
}

fn parse_visibility(vis: &Visibility) -> Result<proc_macro2::TokenStream> {
    Ok(match vis {
        Visibility::Inherited => quote!(pub(super)),
        Visibility::Public(_) => quote!(pub),
        Visibility::Restricted(vis_restricted) => {
            if vis_restricted.in_token.is_some() {
                bail!(
                    vis,
                    "RegMap derive does not support `pub(in ...)` visibilities"
                );
            } else {
                let path = &vis_restricted.path;
                if path.is_ident("crate") {
                    quote!(pub(crate))
                } else if path.is_ident("super") {
                    quote!(pub(in super::super))
                } else if path.is_ident("self") {
                    quote!(pub(super))
                } else {
                    bail!(vis, "RegMap derive found an unexpected visibility");
                }
            }
        }
    })
}

fn is_integer(ident: &Ident) -> bool {
    ident == "u8"
        || ident == "u16"
        || ident == "u32"
        || ident == "u64"
        || ident == "u128"
        || ident == "i8"
        || ident == "i16"
        || ident == "i32"
        || ident == "i64"
        || ident == "i128"
}

mod kw {
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW);
}
#[derive(Default)]
enum RegAccess {
    RO,
    WO,
    #[default]
    RW,
}
impl syn::parse::Parse for RegAccess {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::RO) {
            input.parse::<kw::RO>().map(|_| RegAccess::RO)
        } else if lookahead.peek(kw::WO) {
            input.parse::<kw::WO>().map(|_| RegAccess::WO)
        } else if lookahead.peek(kw::RW) {
            input.parse::<kw::RW>().map(|_| RegAccess::RW)
        } else {
            Err(lookahead.error())
        }
    }
}
impl quote::ToTokens for RegAccess {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            RegAccess::RO => tokens.extend(quote!(::reg_map::access::ReadOnly)),
            RegAccess::WO => tokens.extend(quote!(::reg_map::access::WriteOnly)),
            RegAccess::RW => tokens.extend(quote!(::reg_map::access::ReadWrite)),
        }
    }
}

fn check_repr(input: &DeriveInput) -> Result<()> {
    let mut repr_c = false;
    let mut repr_align = None::<usize>;

    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                // #[repr(C)]
                if meta.path.is_ident("C") {
                    repr_c = true;
                    return Ok(());
                }

                // #[repr(transparent)]
                if meta.path.is_ident("transparent") {
                    // TODO: this is possibly OK, investigate...
                    return Err(meta.error("RegMap derive does not support #[repr(transparent)]"));
                }

                // #[repr(align(N))]
                if meta.path.is_ident("align") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let lit: syn::LitInt = content.parse()?;
                    let n: usize = lit.base10_parse()?;
                    repr_align = Some(n);
                    return Ok(());
                }

                // #[repr(packed)] or #[repr(packed(N))], omitted N means 1
                if meta.path.is_ident("packed") {
                    return Err(meta.error("RegMap derive does not support #[repr(packed)]"));
                }

                Err(meta.error("RegMap derive found an unrecognized #[repr(...)] attribute"))
            })?;
        }
    }

    if repr_c {
        Ok(())
    } else {
        bail!("RegMap derive requires #[repr(C)]")
    }
}

fn parse_field(field: &syn::Field) -> Result<proc_macro2::TokenStream> {
    let name = field.ident.as_ref().expect("struct fields are named");
    let ty = &field.ty;
    let ret_sig = parse_ret_type(field, ty)?;
    let doc = parse_docs(field);
    Ok(match ty {
        Type::Array(TypeArray { .. }) => quote!(
            #doc
            #[inline]
            pub fn #name (&self) -> #ret_sig {
                unsafe { ::reg_map::RegArray::__MACRO_ONLY__from_ptr(::core::ptr::addr_of_mut!((*self.as_ptr()).#name)) }
            }
        ),
        Type::Path(type_path) => {
            let ident = &type_path.path.segments[0].ident;
            if is_integer(ident) {
                quote!(
                    #doc
                    #[inline]
                    pub fn #name (&self) -> #ret_sig {
                        unsafe { ::reg_map::Reg::__MACRO_ONLY__from_ptr(::core::ptr::addr_of_mut!((*self.as_ptr()).#name)) }
                    }
                )
            } else {
                let ptr_ty = Ident::new(&format!("{}Ptr", ident), Span::call_site());
                quote!(
                    #doc
                    #[inline]
                    pub fn #name (&self) -> #ret_sig {
                        unsafe { #ptr_ty::from_ptr(::core::ptr::addr_of_mut!((*self.as_ptr()).#name)) }
                    }
                )
            }
        }
        _ => bail!(
            field,
            "RegMap derive supports only field of type Path or Array"
        ),
    })
}

fn parse_ret_type(field: &syn::Field, ty: &Type) -> Result<proc_macro2::TokenStream> {
    match ty {
        Type::Array(TypeArray { elem, len, .. }) => {
            // recursive!
            let inner_sig = parse_ret_type(field, elem)?;
            Ok(quote!(::reg_map::RegArray<'a, #inner_sig, {#len}>))
        }
        Type::Path(type_path) => {
            let ident = &type_path.path.segments[0].ident;
            if is_integer(ident) {
                let mut access = RegAccess::default();
                for attr in &field.attrs {
                    if attr.path().is_ident("reg") {
                        access = attr.parse_args()?;
                    }
                }
                Ok(quote!(::reg_map::Reg<'a, #ident, #access>))
            } else {
                let ptr_ty = Ident::new(&format!("{}Ptr", ident), Span::call_site());
                Ok(quote!(#ptr_ty<'a>))
            }
        }
        _ => bail!(
            field,
            "RegMap derive supports only field of type Path or Array"
        ),
    }
}
fn parse_docs(field: &syn::Field) -> proc_macro2::TokenStream {
    let mut docs = quote!();
    for attr in &field.attrs {
        if attr.path().is_ident("doc") {
            let text = &attr
                .meta
                .require_name_value()
                .expect("doc attributes are name-value")
                .value;
            docs.extend(quote!(#[doc = #text]));
        }
    }
    docs
}
