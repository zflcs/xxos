extern crate proc_macro;
extern crate alloc;

use proc_macro::TokenStream;
use quote::quote;
use syn;


#[proc_macro_derive(Downcast)]
pub fn downcast_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the trait implementation
    let name = &ast.ident;
    let gen = quote! {
        impl Task for StructName1 {
            fn execute(&self) {
                println!("running...");
            }
        
            fn as_any(&self) -> &dyn Any {
                self
            }
        }
        impl #name {
            fn downcast(task: &dyn Task) -> &#name {
                task.as_any().downcast_ref::<#name>().unwrap()
            }
        }
    };
    gen.into()
}

