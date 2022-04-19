use proc_macro2::TokenStream;
use rstgen::swift::Swift;
use rstgen::{swift, Tokens};

use crate::ast::types::{AstBaseType, AstType};
use crate::base::{Convertible, Direction};
use crate::ident;
use crate::swift::ty::basic::quote_free_swift_ptr;

pub(crate) struct VecStruct {
    pub(crate) ty: AstType,
}

impl VecStruct {
    fn struct_name(&self) -> String {
        match self.ty.clone() {
            AstType::Vec(AstBaseType::Struct(ref origin)) => origin.to_string(),
            _ => "".to_string(),
        }
    }
}

impl<'a> Convertible<Swift<'a>> for VecStruct {
    fn native_to_transferable(
        &self,
        origin: String,
        _direction: Direction,
    ) -> Tokens<'static, Swift<'a>> {
        let mut body = Tokens::new();
        body.append(toks_f!("{{ () -> C{}Array in", self.struct_name()));
        nested_f!(body, |t| {
            nested_f!(
                t,
                "let buffer = UnsafeMutablePointer<Proxy{}>.allocate(capacity: {}.count)",
                self.struct_name(),
                origin
            );
            nested_f!(
                t,
                "{}.map {{ each in each.intoProxy() }}.withUnsafeBufferPointer {{ inner in",
                origin
            );
            nested_f!(t, |tt| {
                nested_f!(
                    tt,
                    "buffer.initialize(from: inner.baseAddress!, count: inner.count)"
                )
            });
            nested_f!(t, "}")
        });
        nested_f!(
            body,
            quote_free_swift_ptr(&format!("Proxy{}", &self.struct_name()))
        );
        nested_f!(
            body,
            "return C{}Array(ptr: buffer, len: Int32({}.count), free_ptr: free_ptr)",
            self.struct_name(),
            origin
        );
        push_f!(body, "}()");
        body
    }

    fn transferable_to_native(
        &self,
        origin: String,
        _direction: Direction,
    ) -> Tokens<'static, Swift<'a>> {
        let mut body = Tokens::new();
        let _proxy_ty = format!("Proxy{}", &self.struct_name());
        let _c_array_ty = format!("C{}Array", &self.struct_name());
        body.append(toks_f!(" {{ () -> [{}] in", self.struct_name()));
        nested_f!(
            body,
            "let proxy_array = Array(UnsafeBufferPointer(start: {}.ptr, count: Int({}.len)))",
            origin,
            origin
        );
        nested_f!(
            body,
            "let struct_arg = proxy_array.map { proxy in DemoStruct(proxy: proxy) }"
        );
        nested_f!(
            body,
            "({}.free_ptr)(UnsafeMutablePointer(mutating:{}.ptr!), {}.len)",
            origin,
            origin,
            origin
        );
        nested_f!(body, "return struct_arg");
        push_f!(body, "}()");
        body
    }

    fn rust_to_transferable(&self, origin: TokenStream, _direction: Direction) -> TokenStream {
        let proxy_struct = ident!(&format!("Proxy{}", &self.struct_name()));
        let struct_array_str = format!("C{}Array", &self.struct_name());
        let struct_array_name = ident!(&struct_array_str);
        let free_proxy_struct_array_fn = ident!(&format!("free_{}", &struct_array_str));
        quote! {{
            let mut tmp_vec = #origin.into_iter().map(|each| #proxy_struct::from(each)).collect::<Vec<#proxy_struct>>();
            tmp_vec.shrink_to_fit();
            let ptr = tmp_vec.as_ptr();
            let len = tmp_vec.len();
            std::mem::forget(tmp_vec);
            #struct_array_name {
                ptr,
                len: len as i32,
                free_ptr: #free_proxy_struct_array_fn
            }
        }}
    }

    fn transferable_to_rust(&self, origin: TokenStream, _direction: Direction) -> TokenStream {
        let proxy_struct = ident!(&format!("Proxy{}", &self.struct_name()));
        quote! {{
            let tmp_vec: Vec<#proxy_struct> = unsafe {
                std::slice::from_raw_parts(#origin.ptr as *mut #proxy_struct, #origin.len as usize).to_vec()
            };
            (#origin.free_ptr)(#origin.ptr as (*mut #proxy_struct), #origin.len);
            tmp_vec.into_iter().map(|each| each.into()).collect()
        }}
    }

    fn native_type(&self) -> Swift<'a> {
        swift::local(format!("[{}]", self.struct_name()))
    }

    fn quote_common_bridge(&self) -> TokenStream {
        quote! {}
    }

    fn quote_common_artifact(&self) -> Tokens<'static, Swift<'static>> {
        Tokens::new()
    }
}