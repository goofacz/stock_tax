use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Currency)]
pub fn derive_currency(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let name = format!("{}", ident).to_uppercase();
    let output = quote! {
        impl Currency for #ident {
            fn get_value(&self) -> f64 {
                self.0
            }

            fn get_name(&self) -> &str {
                return #name;
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Mul)]
pub fn derive_mul(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl<T> Mul<T> for #ident
        where
            f64: From<T>,
        {
            type Output = #ident;
            fn mul(self, rhs: T) -> Self {
                #ident(self.0 * f64::from(rhs))
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Div)]
pub fn derive_div(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl<T> Div<T> for #ident
        where
            f64: From<T>,
        {
            type Output = #ident;
            fn div(self, rhs: T) -> Self {
                #ident(self.0 / f64::from(rhs))
            }
        }
    };
    output.into()
}
