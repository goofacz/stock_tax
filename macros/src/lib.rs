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

        impl Mul<Tax> for #ident {
            type Output = #ident;
            fn mul(self, rhs: Tax) -> Self {
                let result = self.0.checked_mul(rhs.get_value()).unwrap().round_dp(2);
                #ident(result)
            }
        }

        impl Mul<Decimal> for #ident {
            type Output = #ident;
            fn mul(self, rhs: Decimal) -> Self {
                let result = self.0.checked_mul(rhs).unwrap().round_dp(2);
                #ident(result)
            }
        }

        impl Div<Decimal> for #ident {
            type Output = #ident;
            fn div(self, rhs: Decimal) -> Self {
                let result = self.0.checked_div(rhs).unwrap().round_dp(2);
                #ident(result)
            }
        }

        impl fmt::Display for #ident {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} {}", self.get_value(), self.get_code())
            }
        }

        impl<T> Builder<T> for #ident  where
            T: Into<Decimal>
        {
            fn new(amount: T) -> #ident {
                #ident(amount.into())
            }

            fn new_box(amount: T) -> Box<#ident> {
                Box::new(#ident(amount.into()))
            }
        }

        impl<f32> Builder<f32> for #ident {
            fn new(amount: f32) -> #ident {
                #ident(amount.into())
            }

            fn new_box(amount: f32) -> Box<#ident> {
                Box::new(#ident(amount.into()))
            }
        }

        impl #ident {
            pub fn abs(self) -> #ident {
                #ident(self.0.abs())
            }
        }
    };
    output.into()
}
