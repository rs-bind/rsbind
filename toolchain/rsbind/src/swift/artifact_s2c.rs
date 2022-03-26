use rstgen::swift::Swift;
use rstgen::Tokens;

use crate::ast::contract::desc::ArgDesc;
use crate::ast::types::{AstBaseType, AstType};
use crate::errors::*;
use crate::swift::mapping::SwiftMapping;
use crate::swift::types::SwiftType;

///
/// Swift to C data convert.
///
pub(crate) fn fill_arg_convert(method_body: &mut Tokens<Swift>, arg: &ArgDesc) -> Result<()> {
    println!("quote arg convert for {}", arg.name.clone());
    let s_arg_name = format!("s_{}", &arg.name);
    match arg.ty.clone() {
        AstType::Void => {}
        AstType::Boolean => method_body.push(toks!(
            "let ",
            s_arg_name,
            ": Int32 = ",
            arg.name.clone(),
            " ? 1 : 0"
        )),
        AstType::Byte(_)
        | AstType::Short(_)
        | AstType::Int(_)
        | AstType::Long(_)
        | AstType::Float(_)
        | AstType::Double(_) => {
            let ty = SwiftMapping::map_transfer_type(&arg.ty);
            method_body.push(toks!(
                "let ",
                s_arg_name,
                " = ",
                ty,
                "(",
                arg.name.clone(),
                ")"
            ))
        }
        AstType::String => {
            method_body.push(toks!("let ", s_arg_name, " = ", arg.name.clone()))
        }
        AstType::Vec(AstBaseType::Byte(_))
        | AstType::Vec(AstBaseType::Short(_))
        | AstType::Vec(AstBaseType::Int(_))
        | AstType::Vec(AstBaseType::Long(_)) => {
            let arg_buffer_name = format!("{}_buffer", &arg.name);
            let transfer_ty = SwiftMapping::map_transfer_type(&arg.ty);
            method_body.push(toks!(
                "let ",
                s_arg_name,
                " = ",
                transfer_ty,
                "(ptr: ",
                arg_buffer_name.clone(),
                ".baseAddress, len: Int32(",
                arg_buffer_name,
                ".count))"
            ))
        }
        AstType::Vec(AstBaseType::Struct(_)) => {
            method_body.push(toks!("var ", format!("s_{}", &arg.name), ": String?"));
            method_body.push(toks!("autoreleasepool {"));
            let encoder_name = format!("{}_encoder", &arg.name);
            method_body.nested(toks!("let ", encoder_name.clone(), " = JSONEncoder()"));
            method_body.nested(toks!(
                "let ",
                format!("data_{}", &arg.name),
                " = try! ",
                encoder_name,
                ".encode(",
                arg.name.clone(),
                ")"
            ));
            method_body.nested(toks!(
                format!("s_{}", &arg.name),
                " = String(data: ",
                format!("data_{}", &arg.name),
                ", encoding: .utf8)!"
            ));
            method_body.push(toks!("}"));
        }
        AstType::Vec(_) | AstType::Struct(_) => {
            method_body.push(toks!("var ", format!("s_{}", &arg.name), ": String?"));
            method_body.push(toks!("autoreleasepool {"));
            let encoder_name = format!("{}_encoder", &arg.name);
            method_body.nested(toks!("let ", encoder_name.clone(), " = JSONEncoder()"));
            method_body.nested(toks!(
                "let ",
                format!("data_{}", &arg.name),
                " = try! ",
                encoder_name,
                ".encode(",
                arg.name.clone(),
                ")"
            ));
            method_body.nested(toks!(
                format!("s_{}", &arg.name),
                " = String(data: ",
                format!("data_{}", &arg.name),
                ", encoding: .utf8)!"
            ));
            method_body.push(toks!("}"));
        }
        AstType::Callback(_) => {
            println!("argument callback in s2c")
        }
    }

    Ok(())
}

pub(crate) fn fill_return_type_convert(
    method_body: &mut Tokens<Swift>,
    return_type: &AstType,
    crate_name: &str,
) -> Result<()> {
    let crate_name = crate_name.replace('-', "_");
    match return_type.clone() {
        AstType::Void => {}
        AstType::Byte(_)
        | AstType::Short(_)
        | AstType::Int(_)
        | AstType::Long(_)
        | AstType::Float(_)
        | AstType::Double(_) => {
            let ty = SwiftMapping::map_swift_sig_type(return_type);
            method_body.push(toks!("let s_result = ", ty, "(result)"));
        }
        AstType::Boolean => {
            method_body.push(toks!("let s_result = result > 0 ? true : false"));
        }
        AstType::String => {
            method_body.push(toks!("let s_result = String(cString:result!)"));
            method_body.push(toks!(format!(
                "{}_free_str(UnsafeMutablePointer(mutating: result!))",
                &crate_name
            )));
        }
        AstType::Vec(AstBaseType::Byte(_)) => {
            let ty = SwiftMapping::map_swift_sig_type(return_type);
            method_body.push(toks!(
                "let s_result = ",
                ty,
                "(UnsafeBufferPointer(start: result.ptr, count: Int(result.len)))"
            ));
            method_body.push(toks!(
                format!("{}_free_rust", &crate_name),
                "(UnsafeMutablePointer(mutating: result.ptr), UInt32(result.len))"
            ));
        }
        AstType::Vec(AstBaseType::Short(_)) => {
            let ty = SwiftMapping::map_swift_sig_type(return_type);
            method_body.push(toks!(
                "let s_result = ",
                ty,
                "(UnsafeBufferPointer(start: result.ptr, count: Int(result.len)))"
            ));
            method_body.push(toks!("UnsafeMutablePointer(mutating: result.ptr).withMemoryRebound(to: Int8.self, capacity: 2 * Int(result.len)) {"));
            method_body.nested(toks!(
                format!("{}_free_rust", &crate_name),
                "($0, UInt32(2 * result.len))"
            ));
            method_body.push(toks!("}"));
        }
        AstType::Vec(AstBaseType::Int(_)) => {
            let ty = SwiftMapping::map_swift_sig_type(return_type);
            method_body.push(toks!(
                "let s_result = ",
                ty,
                "(UnsafeBufferPointer(start: result.ptr, count: Int(result.len)))"
            ));
            method_body.push(toks!("UnsafeMutablePointer(mutating: result.ptr).withMemoryRebound(to: Int8.self, capacity: 4 * Int(result.len)) {"));
            method_body.nested(toks!(
                format!("{}_free_rust", &crate_name),
                "($0, UInt32(4 * result.len))"
            ));
            method_body.push(toks!("}"));
        }
        AstType::Vec(AstBaseType::Long(_)) => {
            let ty = SwiftMapping::map_swift_sig_type(return_type);
            method_body.push(toks!(
                "let s_result = ",
                ty,
                "(UnsafeBufferPointer(start: result.ptr, count: Int(result.len)))"
            ));
            method_body.push(toks!("UnsafeMutablePointer(mutating: result.ptr).withMemoryRebound(to: Int8.self, capacity: 8 * Int(result.len)) {"));
            method_body.nested(toks!(
                format!("{}_free_rust", &crate_name),
                "($0, UInt32(8 * result.len))"
            ));
            method_body.push(toks!("}"));
        }
        AstType::Vec(_) => {
            let return_ty = SwiftType::new(return_type.clone());
            method_body.push(toks!("let ret_str = String(cString:result!)"));
            method_body.push(toks!(format!(
                "{}_free_str(UnsafeMutablePointer(mutating: result!))",
                &crate_name
            )));
            method_body.push(toks!(
                "var s_tmp_result:",
                Swift::from(return_ty.clone()),
                "?"
            ));
            method_body.push(toks!("autoreleasepool {"));
            method_body.nested(toks!("let ret_str_json = ret_str.data(using: .utf8)!"));
            method_body.nested(toks!("let decoder = JSONDecoder()"));
            method_body.nested(toks!(
                "s_tmp_result = try! decoder.decode(",
                Swift::from(return_ty),
                ".self, from: ret_str_json)"
            ));
            method_body.push(toks!("}"));
            method_body.push(toks!("let s_result = s_tmp_result!"));
        }
        AstType::Callback(_) => {}
        AstType::Struct(struct_name) => {
            method_body.push(toks!("let ret_str = String(cString:result!)"));
            method_body.push(toks!(format!(
                "{}_free_str(UnsafeMutablePointer(mutating: result!))",
                &crate_name
            )));
            method_body.push(toks!("var s_tmp_result: ", struct_name.clone(), "?"));
            method_body.push(toks!("autoreleasepool {"));
            method_body.nested(toks!("let ret_str_json = ret_str.data(using: .utf8)!"));
            method_body.nested(toks!("let decoder = JSONDecoder()"));
            method_body.nested(toks!(
                "s_tmp_result = try! decoder.decode(",
                struct_name,
                ".self, from: ret_str_json)"
            ));
            method_body.push(toks!("}"));
            method_body.push(toks!("let s_result = s_tmp_result!"));
        }
    }

    match return_type.clone() {
        AstType::Void => {}
        _ => method_body.push(toks!("return s_result")),
    }
    Ok(())
}