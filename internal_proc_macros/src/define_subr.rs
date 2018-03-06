use syn;
use quote;
use regex;
// use syn::token::{Semi};
use syn::synom::{Synom};
use syn::punctuated::Punctuated;
use quote::{ToTokens, Tokens};
use proc_macro2::{TokenStream};

struct ReqArg { name: syn::Ident, typ: syn::TypePath }
struct OptArg { pub arg: ReqArg }
trait Arg {
    fn get_name(&self) -> &syn::Ident;
    fn get_type(&self) -> &syn::TypePath;
    fn get_conv(&self) -> Tokens;
}

impl Arg for ReqArg {
    fn get_name(&self) -> &syn::Ident { &self.name }
    fn get_type(&self) -> &syn::TypePath { &self.typ }

    fn get_conv(&self) -> Tokens {
        let name = self.get_name();
        quote!(
            Scm::<Untyped>::from_raw(#name).into_type().unwrap();
        )
    }
}

impl Arg for OptArg {
    fn get_name(&self) -> &syn::Ident { self.arg.get_name() }
    fn get_type(&self) -> &syn::TypePath { self.arg.get_type() }

    fn get_conv(&self) -> Tokens {
        quote!(
            // TODO: figure out how to convert SCM to Option<Scm<T>>
        )
    }
}

impl Synom for ReqArg {
    named!(parse -> Self, do_parse!(
            name: syn!(syn::Ident) >>
            punct!(:) >>
            typ: syn!(syn::TypePath) >>
            cond_reduce!(
                typ.path.segments.last().unwrap().value().ident == "Scm") >>

            (ReqArg { name, typ })
    ));
}

impl Synom for OptArg {
    named!(parse -> Self, do_parse!(
            name: syn!(syn::Ident) >>
            punct!(:) >>

            _opt: syn!(syn::Ident) >>
            cond_reduce!(_opt == "Option") >>

            punct!(<) >>
            typ: syn!(syn::TypePath) >>
            cond_reduce!(
                typ.path.segments.last().unwrap().value().ident == "Scm") >>
            punct!(>) >>

            (OptArg { arg: ReqArg { name, typ } })
    ));
}

named!(pub parse_guile_define_subr -> TokenStream, do_parse!(
        name: syn!(syn::Expr) >>
        punct!(,) >>
        punct!(|) >>
        args:  call!(Punctuated::<ReqArg, Token![,]>::parse_terminated) >>
        oargs: call!(Punctuated::<OptArg, Token![,]>::parse_terminated) >>
        punct!(|) >>
        punct!(->) >>
        ret: syn!(syn::TypePath) >>
        cond_reduce!(
            ret.path.segments.last().unwrap().value().ident == "Scm") >>
        body: syn!(syn::Expr) >>

        ({
            let arg_names  = args.iter().map(|ref a| a.get_name())
                .collect::<Vec<_>>();
            let arg_names2 = arg_names.clone();
            let arg_types  = args.iter().map(|ref a| a.get_type());
            let arg_convs  = args.iter().map(|ref a| a.get_conv());
            let num_args   = args.len();

            let oarg_names = oargs.iter().map(|ref a| a.get_name())
                .collect::<Vec<_>>();
            let oarg_names2 = oarg_names.clone();
            let oarg_types = oargs.iter().map(|ref a| a.get_type());
            let oarg_convs = oargs.iter().map(|ref a| a.get_conv());
            let num_oargs  = oargs.len();

            quote!({
                unsafe {
                    unsafe extern "C" ___tmp_c_fn(#(#arg_names: SCM),* , #(#oarg_names: SCM),* ) -> SCM {
                        #(let #arg_names2:  #arg_types  = #arg_convs; )*
                        #(let #oarg_names2: #oarg_types = #oarg_convs;)*

                        let o: #ret = #body;

                        o.into_raw()
                    }

                    let _ = scm_c_define_gsubr(
                        CString::new(#name).unwrap().as_ptr(),
                        #num_args, #num_oargs, 0,
                        ___tmp_c_fn as scm_t_subr
                        );
                }
            }).into()
        })
));
