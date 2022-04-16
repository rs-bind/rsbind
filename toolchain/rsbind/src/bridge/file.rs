use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use proc_macro2::{Ident, TokenStream};

use crate::ast::contract::desc::*;
use crate::ast::imp::desc::*;
use crate::ast::types::*;
use crate::errors::ErrorKind::*;
use crate::errors::*;
use crate::ident;

struct GenResult {
    pub result: Result<TokenStream>,
}

pub(crate) enum TypeDirection {
    Argument,
    Return,
}
///
/// Executor for generationg core files of bridge mod.
///
pub(crate) struct BridgeFileGen<'a, T: FileGenStrategy> {
    pub out_dir: &'a Path,
    pub traits: &'a [TraitDesc],
    pub structs: &'a [StructDesc],
    pub imps: &'a [ImpDesc],
    pub strategy: T,
}

///
/// Strategy for generating core files in bridge mod.
///
pub(crate) trait FileGenStrategy {
    fn gen_sdk_file(&self, mod_names: &[String]) -> Result<TokenStream>;
    fn quote_common_use_part(&self) -> Result<TokenStream>;
    fn quote_common_part(&self, trait_desc: &[TraitDesc]) -> Result<TokenStream>;
    fn quote_for_all_cb(&self, callbacks: &[&TraitDesc]) -> Result<TokenStream>;
    fn quote_callback_structures(
        &self,
        callback: &TraitDesc,
        callbacks: &[&TraitDesc],
    ) -> Result<TokenStream>;
    fn quote_for_structures(&self, struct_desc: &StructDesc) -> Result<TokenStream>;
    fn quote_method_sig(
        &self,
        trait_desc: &TraitDesc,
        impl_desc: &ImpDesc,
        method: &MethodDesc,
        callbacks: &[&TraitDesc],
        structs: &[StructDesc],
    ) -> Result<TokenStream>;
    fn quote_arg_convert(
        &self,
        trait_desc: &TraitDesc,
        args: &ArgDesc,
        callbacks: &[&TraitDesc],
    ) -> Result<TokenStream>;
    fn quote_return_convert(
        &self,
        trait_desc: &TraitDesc,
        callbacks: &[&TraitDesc],
        return_ty: &AstType,
        ret_name: &str,
    ) -> Result<TokenStream>;
    fn ty_to_tokens(&self, ast_type: &AstType, direction: TypeDirection) -> Result<TokenStream>;
}

impl<'a, T: FileGenStrategy + 'a> BridgeFileGen<'a, T> {
    ///
    /// generate sdk.rs files
    ///
    pub(crate) fn gen_sdk_file(&self, file_name: &str, mod_names: &[String]) -> Result<()> {
        let result = self.strategy.gen_sdk_file(mod_names).unwrap();

        let out_file_path = self.out_dir.join(file_name);
        let mut f = File::create(&out_file_path).unwrap();
        f.write_all(&result.to_string().into_bytes()).unwrap();

        Ok(())
    }

    ///
    /// generate one bridge file for one contract mod.
    ///
    pub(crate) fn gen_one_bridge_file(&self, file_name: &str) -> Result<()> {
        println!("[bridge][{}]  🔆  begin generate bridge file.", file_name);
        let use_part = self.quote_use_part().unwrap();
        let common_part = self.strategy.quote_common_part(self.traits).unwrap();
        let bridge_codes = self.gen_for_one_mod().unwrap();

        let mut merge_tokens = quote! {
            #use_part
            #common_part
        };

        for bridge_code in bridge_codes {
            if let Ok(code) = bridge_code.result {
                merge_tokens = quote! {
                    #merge_tokens
                    #code
                };
            }
        }

        let out_file_path = self.out_dir.join(file_name);
        let mut f = File::create(&out_file_path).unwrap();
        f.write_all(&merge_tokens.to_string().into_bytes()).unwrap();

        println!("[bridge][{}]  ✅  end generate bridge file.", file_name);
        Ok(())
    }

    ///
    /// generate bridge file from a file of trait.
    ///
    fn gen_for_one_mod(&self) -> Result<Vec<GenResult>> {
        let mut results: Vec<GenResult> = vec![];

        let callbacks = self
            .traits
            .iter()
            .filter(|desc| desc.is_callback)
            .collect::<Vec<&TraitDesc>>();

        println!("callbacks is {:?}", &callbacks);

        for desc in self.traits.iter() {
            if desc.is_callback {
                results.push(GenResult {
                    result: self.strategy.quote_callback_structures(desc, &callbacks),
                });
                continue;
            }

            let imps = self
                .imps
                .iter()
                .filter(|info| info.contract == desc.name)
                .collect::<Vec<&ImpDesc>>();

            println!("desc => {:?}", desc);
            println!("imps => {:?}", imps);
            println!("all imps => {:?}", &self.imps);

            match imps.len().cmp(&1) {
                Ordering::Less => {}
                Ordering::Equal => {
                    results.push(GenResult {
                        result: self.generate_for_one_trait(
                            desc,
                            imps[0],
                            &callbacks,
                            self.structs,
                        ),
                    });
                }
                Ordering::Greater => {
                    println!("You have more than one impl for trait {}", desc.name);
                    return Err(GenerateError(format!(
                        "You have more than one impl for trait {}",
                        desc.name
                    ))
                    .into());
                }
            }
        }

        let tokens = self.strategy.quote_for_all_cb(&callbacks);
        results.push(GenResult { result: tokens });

        for struct_desc in self.structs.iter() {
            let tokens = self.strategy.quote_for_structures(struct_desc);
            results.push(GenResult { result: tokens });
        }

        Ok(results)
    }

    fn generate_for_one_trait(
        &self,
        trait_desc: &TraitDesc,
        imp: &ImpDesc,
        callbacks: &[&TraitDesc],
        structs: &[StructDesc],
    ) -> Result<TokenStream> {
        println!(
            "[bridge][{}]  🔆  begin generate bridge on trait.",
            &trait_desc.name
        );
        let mut merge: TokenStream = TokenStream::new();

        for method in trait_desc.methods.iter() {
            println!(
                "[bridge][{}.{}]  🔆  begin generate bridge method.",
                &trait_desc.name, &method.name
            );
            let one_method = self
                .quote_one_method(trait_desc, imp, method, callbacks, structs)
                .unwrap();

            println!(
                "[bridge][{}.{}]  ✅  end generate bridge method.",
                &trait_desc.name, &method.name
            );

            merge = quote! {
                #merge
                #one_method
            };
        }
        println!(
            "[bridge][{}]  ✅  end generate bridge on trait.",
            &trait_desc.name
        );
        Ok(merge)
    }

    ///
    /// quote use part
    ///
    fn quote_use_part(&self) -> Result<TokenStream> {
        println!("[bridge]  🔆  begin quote use part.");
        let mut merge = self.strategy.quote_common_use_part().unwrap();

        for trait_desc in self.traits.iter() {
            if trait_desc.is_callback {
                println!("Skip callback trait {}", &trait_desc.name);
                continue;
            }

            let imps = self
                .imps
                .iter()
                .filter(|info| info.contract == trait_desc.name)
                .collect::<Vec<&ImpDesc>>();

            match imps.len().cmp(&1) {
                Ordering::Less => {}
                Ordering::Equal => {
                    let use_part = self
                        .quote_one_use_part(&trait_desc.mod_path, &imps[0].mod_path)
                        .unwrap();
                    merge = quote! {
                       #use_part
                       #merge
                    };
                }
                Ordering::Greater => {
                    println!("You have more than one impl for trait {}", trait_desc.name);
                    return Err(GenerateError(format!(
                        "You have more than one impl for trait {}",
                        trait_desc.name
                    ))
                    .into());
                }
            }
        }
        println!("[bridge]  ✅  end quote use part.");
        Ok(merge)
    }

    fn quote_one_use_part(&self, trait_mod_path: &str, imp_mod_path: &str) -> Result<TokenStream> {
        let trait_mod_splits: Vec<Ident> = trait_mod_path
            .split("::")
            .collect::<Vec<&str>>()
            .iter()
            .map(|str| ident!(str))
            .collect();
        let imp_mod_splits: Vec<Ident> = imp_mod_path
            .split("::")
            .collect::<Vec<&str>>()
            .iter()
            .map(|str| ident!(str))
            .collect();

        Ok(quote! {
            use #(#trait_mod_splits::)**;
            use #(#imp_mod_splits::)**;
        })
    }

    ///
    /// quote one method
    ///
    fn quote_one_method(
        &self,
        trait_desc: &TraitDesc,
        imp: &ImpDesc,
        method: &MethodDesc,
        callbacks: &[&TraitDesc],
        structs: &[StructDesc],
    ) -> Result<TokenStream> {
        println!(
            "[bridge][{}.{}]  🔆 ️begin quote method.",
            &trait_desc.name, &method.name
        );
        let sig_define = self
            .strategy
            .quote_method_sig(trait_desc, imp, method, callbacks, structs)
            .unwrap();

        let mut arg_convert = TokenStream::new();
        for arg in method.args.iter() {
            let arg_tokens = self
                .strategy
                .quote_arg_convert(trait_desc, arg, callbacks)
                .unwrap();
            arg_convert = quote! {
                #arg_convert
                #arg_tokens
            }
        }

        let call_imp = self.quote_imp_call(&imp.name, method)?;

        let return_handle = self.strategy.quote_return_convert(
            trait_desc,
            callbacks,
            &method.return_type,
            "result",
        )?;

        // combine all the parts
        let result = quote! {
            #sig_define {
                #arg_convert
                #call_imp
                #return_handle
            }
        };

        println!(
            "[bridge][{}.{}] ✅ end quote method.",
            &trait_desc.name, &method.name
        );
        Ok(result)
    }

    fn quote_imp_call(&self, impl_name: &str, method: &MethodDesc) -> Result<TokenStream> {
        println!(
            "[bridge][{}.{}]  🔆 ️begin quote imp call.",
            impl_name, &method.name
        );

        let ret_name_str = "result";
        let imp_fun_name = ident!(&method.name);
        let ret_name_ident = ident!(ret_name_str);

        let tmp_arg_names = method
            .args
            .iter()
            .map(|e| &e.name)
            .map(|arg_name| ident!(&format!("r_{}", arg_name)))
            .collect::<Vec<Ident>>();

        let rust_args_repeat = quote! {
            #(#tmp_arg_names),*
        };

        let imp_ident = ident!(impl_name);
        let imp_call = match method.return_type.clone() {
            AstType::Void => quote! {
                let #ret_name_ident = #imp_ident::#imp_fun_name(#rust_args_repeat);
            },
            AstType::Vec(AstBaseType::Byte(_))
            | AstType::Vec(AstBaseType::Short(_))
            | AstType::Vec(AstBaseType::Int(_))
            | AstType::Vec(AstBaseType::Long(_)) => {
                quote! {
                    let mut #ret_name_ident = #imp_ident::#imp_fun_name(#rust_args_repeat);
                }
            }
            AstType::Vec(_)
            | AstType::Struct(_)
            | AstType::Callback(_)
            | AstType::String
            | AstType::Byte(_)
            | AstType::Short(_)
            | AstType::Int(_)
            | AstType::Long(_)
            | AstType::Float(_)
            | AstType::Double(_)
            | AstType::Boolean => {
                quote! {
                    let #ret_name_ident = #imp_ident::#imp_fun_name(#rust_args_repeat);
                }
            }
        };

        println!(
            "[bridge][{}.{}]  ✅ end quote imp call.",
            impl_name, &method.name
        );

        Ok(imp_call)
    }
}
