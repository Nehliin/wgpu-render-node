extern crate proc_macro;
use proc_macro_error::{abort, proc_macro_error};
use proc_macro::TokenStream;
use quote::quote;


#[proc_macro_error]
#[proc_macro_derive(GpuData)]
pub fn gpu_data_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let stream: TokenStream  = quote! {#[repr(C)] struct Dummy;}.into();
    let repr_c_tokens: syn::DeriveInput = syn::parse(stream).unwrap();
    if !ast.attrs.iter().any(|attr| attr == &repr_c_tokens.attrs[0]) {
        abort! {
            name,
            format!("Invalid ABI guarentee for {} struct", name); 
            note = "All GpuData must #[repr(C)]"; 
            help = "Add #[repr(C)] to your GpuData struct";
        };
    }
    let gen = quote! {
        unsafe impl GpuData for #name {}
    };
    gen.into()
}


