use proc_macro2::{Ident, Span, TokenStream};

///
/// Java to Rust data convert.
///
///
use crate::ast::contract::desc::{ArgDesc, TraitDesc};
use crate::ast::types::{AstBaseType, AstType};
use crate::bridge::file::{TMP_ARG_PREFIX, TypeDirection};
use crate::ErrorKind::GenerateError;
use crate::errors::*;

pub(crate) fn quote_arg_convert(
    arg: &ArgDesc,
    namespace: &str,
    trait_desc: &TraitDesc,
) -> Result<TokenStream> {
    let rust_arg_name = Ident::new(
        &format!("{}_{}", TMP_ARG_PREFIX, &arg.name),
        Span::call_site(),
    );
    let arg_name_ident = Ident::new(&arg.name, Span::call_site());
    let _class_name = format!("{}.{}", namespace, &trait_desc.name).replace('.', "/");

    let result = match arg.clone().ty {
        AstType::Byte(origin)
        | AstType::Short(origin)
        | AstType::Int(origin)
        | AstType::Long(origin)
        | AstType::Float(origin)
        | AstType::Double(origin) => {
            let origin_type_ident = Ident::new(&origin, Span::call_site());
            quote! {
                let #rust_arg_name = #arg_name_ident as #origin_type_ident;
            }
        }
        AstType::Boolean => {
            quote! {
                let #rust_arg_name = if #arg_name_ident > 0 {true} else {false};
            }
        }
        AstType::String => {
            quote! {
                let #rust_arg_name: String = env.get_string(#arg_name_ident).expect("Couldn't get java string!").into();
            }
        }
        AstType::Vec(AstBaseType::Byte(origin)) => {
            if origin.contains("i8") {
                let tmp_arg_name = Ident::new(&format!("tmp_{}", &arg.name), Span::call_site());
                let tmp_arg_ptr = Ident::new(&format!("tmp_{}_ptr", &arg.name), Span::call_site());
                let tmp_arg_len = Ident::new(&format!("tmp_{}_len", &arg.name), Span::call_site());
                let tmp_arg_cap = Ident::new(&format!("tmp_{}_cap", &arg.name), Span::call_site());
                quote! {
                    let mut #tmp_arg_name = env.convert_byte_array(#arg_name_ident).unwrap();
                    let #tmp_arg_ptr = #tmp_arg_name.as_mut_ptr();
                    let #tmp_arg_len = #tmp_arg_name.len();
                    let #tmp_arg_cap = #tmp_arg_name.capacity();
                    let #rust_arg_name = unsafe {
                        std::mem::forget(#tmp_arg_name);
                        Vec::from_raw_parts(#tmp_arg_ptr as (* mut i8), #tmp_arg_len, #tmp_arg_cap)
                    };
                }
            } else {
                quote! {
                    let #rust_arg_name = env.convert_byte_array(#arg_name_ident).unwrap();
                }
            }
        }
        AstType::Vec(AstBaseType::Struct(origin)) => {
            let json_arg_ident = Ident::new(&format!("json_{}", &arg.name), Span::call_site());
            let tmp_arg_ident = Ident::new(&format!("tmp_{}", &arg.name), Span::call_site());
            let struct_name = Ident::new(&format!("Struct_{}", &origin), Span::call_site());
            let real_struct_name = Ident::new(&origin, Span::call_site());
            quote! {
                let #json_arg_ident: String = env.get_string(#arg_name_ident).expect("Couldn't get java string!").into();
                let #tmp_arg_ident: Vec<#struct_name> = serde_json::from_str(&#json_arg_ident).unwrap();
                let #rust_arg_name: Vec<#real_struct_name> = #tmp_arg_ident.into_iter().map(|each| #real_struct_name::from(each)).collect();
            }
        }
        AstType::Vec(_) => {
            let json_arg_ident = Ident::new(&format!("json_{}", &arg.name), Span::call_site());
            quote! {
                let #json_arg_ident: String = env.get_string(#arg_name_ident).expect("Couldn't get java string!").into();
                let #rust_arg_name = serde_json::from_str(&#json_arg_ident).unwrap();
            }
        }
        AstType::Callback(_) => {
            // Will handle in other places.
            quote! {}
        }
        AstType::Struct(origin) => {
            let json_arg_ident = Ident::new(&format!("json_{}", &arg.name), Span::call_site());
            let tmp_arg_ident = Ident::new(&format!("tmp_{}", &arg.name), Span::call_site());
            let struct_name = Ident::new(&format!("Struct_{}", &origin), Span::call_site());
            let real_struct_name = Ident::new(&origin, Span::call_site());
            quote! {
                let #json_arg_ident: String = env.get_string(#arg_name_ident).expect("Couldn't get java string!").into();
                let #tmp_arg_ident: #struct_name = serde_json::from_str(&#json_arg_ident).unwrap();
                let #rust_arg_name: #real_struct_name = #tmp_arg_ident.into();
            }
        }
        AstType::Void => {
            return Err(
                GenerateError(format!("find unsupported type in arg, {:?}", &arg.ty)).into(),
            );
        }
    };
    Ok(result)
}

pub(crate) fn quote_return_convert(
    return_ty: &AstType,
    _trait_desc: &TraitDesc,
    _callbacks: &[&TraitDesc],
    ret_name: &str,
) -> Result<TokenStream> {
    let ret_name_ident = Ident::new(ret_name, Span::call_site());

    let result = match return_ty.clone() {
        AstType::Void => quote!(),
        AstType::Boolean => quote! {
            if #ret_name_ident {1} else {0}
        },
        AstType::String => quote! {
            env.new_string(#ret_name_ident).expect("Couldn't create java string").into_inner()
        },
        AstType::Vec(AstBaseType::Struct(struct_name)) => {
            let struct_ident = Ident::new(&format!("Struct_{}", &struct_name), Span::call_site());
            quote! {
                let ret_value = #ret_name_ident.into_iter().map(|each| #struct_ident::from(each)).collect::<Vec<#struct_ident>>();
                let json_ret = serde_json::to_string(&ret_value);
                env.new_string(json_ret.unwrap()).expect("Couldn't create java string").into_inner()
            }
        }
        AstType::Vec(AstBaseType::Byte(origin)) => {
            if origin.contains("i8") {
                quote! {
                    let ret_value_ptr = #ret_name_ident.as_mut_ptr();
                    let ret_value_len = #ret_name_ident.len();
                    let ret_value_cap = #ret_name_ident.capacity();
                    let tmp_ret_name = unsafe {
                        std::mem::forget(#ret_name_ident);
                        Vec::from_raw_parts(ret_value_ptr as (* mut u8), ret_value_len, ret_value_cap)
                    };
                    env.byte_array_from_slice(&tmp_ret_name).unwrap()
                }
            } else {
                quote! {
                    env.byte_array_from_slice(&#ret_name_ident).unwrap()
                }
            }
        }
        AstType::Vec(_) => {
            quote! {
                let json_ret = serde_json::to_string(&#ret_name_ident);
                env.new_string(json_ret.unwrap()).expect("Couldn't create java string").into_inner()
            }
        }
        AstType::Struct(name) => {
            let struct_copy_name = Ident::new(&format!("Struct_{}", name), Span::call_site());
            quote! {
                let json_ret = serde_json::to_string(&#struct_copy_name::from(#ret_name_ident));
                env.new_string(json_ret.unwrap()).expect("Couldn't create java string").into_inner()
            }
        }
        _ => {
            let ty_ident = ty_to_tokens(return_ty, TypeDirection::Return).unwrap();
            quote! {
                #ret_name_ident as #ty_ident
            }
        }
    };
    Ok(result)
}

pub(crate) fn ty_to_tokens(ast_type: &AstType, direction: TypeDirection) -> Result<TokenStream> {
    Ok(match ast_type.clone() {
        AstType::Byte(_) => quote!(i8),
        AstType::Short(_) => quote!(i16),
        AstType::Int(_) => quote!(i32),
        AstType::Long(_) => quote!(i64),
        AstType::Float(_) => quote!(f32),
        AstType::Double(_) => quote!(f64),
        AstType::Boolean => quote!(u8),
        AstType::String => match direction {
            TypeDirection::Argument => quote!(JString),
            TypeDirection::Return => quote!(jstring),
        },
        AstType::Vec(base) => match direction {
            TypeDirection::Argument => match base {
                AstBaseType::Byte(_) => {
                    quote!(jbyteArray)
                }
                _ => quote!(JString),
            },
            TypeDirection::Return => match base {
                AstBaseType::Byte(_) => {
                    quote!(jbyteArray)
                }
                _ => quote!(jstring),
            },
        },
        AstType::Struct(_) => match direction {
            TypeDirection::Argument => quote!(JString),
            TypeDirection::Return => quote!(jstring),
        },
        AstType::Callback(_) => quote!(i64),
        AstType::Void => quote!(()),
    })
}