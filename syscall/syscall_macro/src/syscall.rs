use bae::FromAttributes;
use syn;

#[derive(Default, FromAttributes, Debug)]
pub struct Arguments {
    pub args: Option<syn::LitStr>,
}