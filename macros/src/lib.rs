use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Currency)]
pub fn derive_currency(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let code = format_ident!("{}", ident.to_string().to_uppercase());
    let output = quote! {
        impl Currency for #ident {
            fn get_value(&self) -> &Decimal {
                &self.0
            }

            fn get_code(&self) -> Code {
                Code::#code
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Mul)]
pub fn derive_mul(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl Mul<Decimal> for #ident {
            type Output = #ident;
            fn mul(self, rhs: Decimal) -> Self {
                let result = self.0.checked_mul(rhs).unwrap().round_dp(2);
                #ident(result)
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Div)]
pub fn derive_div(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl Div<Decimal> for #ident {
            type Output = #ident;
            fn div(self, rhs: Decimal) -> Self {
                let result = self.0.checked_div(rhs).unwrap().round_dp(2);
                #ident(result)
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Display)]
pub fn derive_display(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl fmt::Display for #ident {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} {}", self.get_value(), self.get_code())
            }
        }
    };
    output.into()
}
