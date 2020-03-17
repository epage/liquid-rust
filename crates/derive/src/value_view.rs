use proc_macro2::*;
use proc_quote::*;
use syn::*;

pub fn derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::liquid::ValueView for #ident #ty_generics #where_clause {
            fn as_debug(&self) -> &dyn ::std::fmt::Debug {
                self
            }

            fn render(&self) -> ::liquid::value::values::DisplayCow<'_> {
                ::liquid::value::values::DisplayCow::Owned(Box::new(::liquid::value::object::ObjectRender::new(self)))
            }
            fn source(&self) -> ::liquid::value::values::DisplayCow<'_> {
                ::liquid::value::values::DisplayCow::Owned(Box::new(::liquid::value::object::ObjectSource::new(self)))
            }
            fn type_name(&self) -> &'static str {
                "object"
            }
            fn query_state(&self, state: ::liquid::value::values::State) -> bool {
                match state {
                    ::liquid::value::values::State::Truthy => true,
                    ::liquid::value::values::State::DefaultValue |
                    ::liquid::value::values::State::Empty |
                    ::liquid::value::values::State::Blank => self.size() == 0,
                }
            }

            fn to_kstr(&self) -> ::kstring::KStringCow<'_> {
                let s = ::liquid::value::object::ObjectRender::new(self).to_string();
                ::kstring::KStringCow::from_string(s)
            }
            fn to_value(&self) -> ::liquid::value::Value {
                ::liquid::value::to_value(self).unwrap()
            }

            fn as_object(&self) -> Option<&dyn ::liquid::ObjectView> {
                Some(self)
            }
        }
    }
}
