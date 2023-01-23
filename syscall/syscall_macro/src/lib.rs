
mod syscall;

extern crate alloc;
extern crate proc_macro;
use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syscall::Arguments;

// SyscallMacro 定义
#[proc_macro_derive(SyscallMacro, attributes(arguments))]
pub fn syscall_macro_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    // ident 当前枚举名称
    let DeriveInput { ident, .. } = input;
    let mut comment_arms = Vec::new();
    if let syn::Data::Enum(syn::DataEnum { variants, .. }) = input.data {
        for variant in variants {
            // 当前枚举项名称如 Alex, Box
            let ident_item = &variant.ident;
            if let Ok(args) = Arguments::from_attributes(&variant.attrs) {
                // 获取属性中定义的参数信息
                let args_str = args.args.unwrap().value();
                let args_vec: Vec<syn::Ident> = args_str.split(", ").map(|s| syn::Ident::new(s, Span::call_site())).collect();
                let len = args_vec.len();
                let syscall_fn = quote::format_ident!("syscall{}", len);
                eprintln!("{}", quote!(
                    #syscall_fn(#ident::#ident_item as usize, #($#args_vec),*)
                ));
                let mut doc = String::from("参数类型为 ");
                for  idx in 0..(len - 1) {
                    doc.push_str(&args_vec[idx].to_string().as_str());
                    doc.push_str(": usize, ");
                }
                doc.push_str(&args_vec[len - 1].to_string().as_str());
                doc.push_str(": usize");
                eprintln!("{}", doc);
                // 生成对应的宏
                comment_arms.push(quote! (
                    #[doc = #doc]
                    #[macro_export]
                    macro_rules ! #ident_item {
                        (#($#args_vec: expr),*) => {
                            unsafe {
                                #syscall_fn(#ident::#ident_item as usize, #($#args_vec),*)
                            }
                            // log::error!("{}", concat!(#($#args_vec),*));
                        }
                    }
                ));
            } else {
                comment_arms.push(quote! ( 
                    #[macro_export]
                    macro_rules ! #ident_item {
                        () => {
                            unsafe {
                                syscall0(#ident::#ident_item as usize)
                            }
                        }
                    }
                ));
            }
            
        }
    }
    quote!(#(#comment_arms)*).into()
}