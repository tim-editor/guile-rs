use std::collections::HashSet;

use syn;
use quote;
use regex;
use syn::token::{Semi, Comma};
use syn::synom::{Synom};
use syn::punctuated::Punctuated;
use quote::{ToTokens, Tokens};
use proc_macro2::{TokenStream};

use regex::Regex;

pub enum ArgDef {
    Type(syn::Type),
    TypeBound(syn::Type, syn::TypeParam),

    SelfRef,
    SelfRefMut,
    SelfMove,

    DisableSelf,
}

impl ArgDef {
    pub fn is_disable_self(&self) -> bool {
        if let &ArgDef::DisableSelf = self { true }
        else { false }
    }
    pub fn is_type_bound(&self) -> bool {
        if let &ArgDef::TypeBound(_, _) = self { true }
        else { false }
    }
    pub fn is_self(&self) -> bool {
        match *self {
            ArgDef::SelfRef
            | ArgDef::SelfRefMut
            | ArgDef::SelfMove => true,

            _ => false,
        }
    }
}

impl Synom for ArgDef {
    named!(parse -> Self, alt!(
            // @_
            tuple!(punct!(@), punct!(_)) => {
                |_| ArgDef::DisableSelf
            }
            |
            // &mut self
            tuple!(punct!(&), keyword!(mut), keyword!(self)) => {
                |_| ArgDef::SelfRefMut
            }
            |
            // &self
            tuple!(punct!(&), keyword!(self)) => {
                |_| ArgDef::SelfRef
            }
            |
            // self
            tuple!(keyword!(self)) => {|_| ArgDef::SelfMove }
            |
            // generic param: `Scm<T>|T: TypeSpec`
            do_parse!(type_: syn!(syn::Type) >> punct!(|) >>
                      tparam: syn!(syn::TypeParam) >>

                      (ArgDef::TypeBound(type_, tparam)))
            |
            // concrete type: `Scm<String>`
            syn!(syn::Type) => { |type_| ArgDef::Type(type_) }
    ));
}


#[derive(Debug)]
#[derive(Clone)]
pub enum CArgDef {
    Immediate(syn::Expr),
    Rest(bool),
}

impl Synom for CArgDef {
    named!(parse -> Self, alt!(
            // expr
            syn!(syn::Expr) => { |e| CArgDef::Immediate(e) }
            |
            // !*
            tuple!(punct!(@), punct!(*), option!(punct!(#))) => {
                |(_, _, r)| { CArgDef::Rest(r.is_none()) }
            }
    ));
}

/// Expand all the `@` flags in the cargs expect
fn expand_carg_flags<T: Synom>(tts: TokenStream) -> T {
    let out = tts.to_string()
        .replace("@ s", "self.data")
        .replace("@*", "! *")
        .replace("@ *", "! *");
    let re = Regex::new(r"@ ([0-9]+) #").unwrap();

    let out = re.replace_all(&out,
                             |c: &regex::Captures| {
                                 format!("a{}.data", &c[1])
                             })
        .to_string()
        .replace("@ ", "a")
        .replace("! *", "@*");

    // eprintln!("EXPANDED: `{}`", out);
    syn::parse_str(&out).expect("Applying @ expansions")
}

#[derive(Debug)]
struct CArgs { pub args: Vec<CArgDef> }
impl Synom for CArgs {
    named!(parse -> Self, do_parse!(
            args: map!(call!(
                    Punctuated::<CArgDef, Comma>::parse_terminated),
                    |es| es.into_iter().collect::<Vec<CArgDef>>()
                    ) >>
            (Self {args: args})
    ));
}

pub struct GuileDef {
    pub attrs: Vec<syn::Attribute>,
    pub public: bool,
    pub name: syn::Ident,
    pub self_disabled: bool,
    pub args: Vec<ArgDef>,
    pub cfunc: syn::Ident,
    pub cargs: Vec<CArgDef>,
    pub ret_from_raw: bool,
    pub ret_ty: Option<syn::Type>,
}

impl Synom for GuileDef {
    named!(parse -> Self, do_parse!(
            attrs:  many0!(call!(syn::Attribute::parse_outer)) >>
            public: alt!(keyword!(pub) => {|_| true}
                         |
                         epsilon!()    => {|_| false}) >>
            keyword!(fn) >>
            name: syn!(syn::Ident) >>
            args: map!(
                parens!(call!(
                        Punctuated::<ArgDef, Comma>::parse_terminated)),
                        |paren| -> Vec<ArgDef> {
                            paren.1.into_iter().collect()
                        }) >>

            cfunc_call: do_parse!(
                       punct!(=>) >>
                       cfunc: syn!(syn::Ident) >>
                       cargs: map!(
                           parens!(syn!(TokenStream)),
                           |p| {
                               expand_carg_flags::<CArgs>(p.1).args
                           }) >>
                       (cfunc, cargs)) >>

            ret_info: map!(option!(do_parse!(
                       punct!(->) >>
                       ir: map!(
                           option!(tuple!(
                                   punct!(@), syn!(syn::Ident)
                                   )),
                           |o| {
                               o.is_some()
                                   && o.unwrap().1.as_ref() == "r"
                           }) >>

                       rt: syn!(syn::Type) >>
                       (ir, Some(rt)))),

                       |ri| if ri.is_none() {
                           (false, None)
                       } else {
                           ri.unwrap()
                       })>>

            (Self {
                attrs:         attrs,
                public:        public,
                name:          name,
                self_disabled: args.iter().any(|a|
                                               a.is_disable_self()
                                               || a.is_self()
                                               ),

                args:          args.into_iter()
                                   .filter(|a|
                                           !a.is_disable_self()
                                           )
                                   .collect(),

                cfunc:         cfunc_call.0,
                cargs:         cfunc_call.1,

                ret_from_raw: ret_info.0,
                ret_ty:       ret_info.1,
            })
    ));
}

impl GuileDef {
    pub fn construct(&self) -> quote::Tokens {
        let mut tokens = if self.public {
            quote!(pub)
        } else {
            quote::Tokens::new()
        };

        let attrs  = &self.attrs;
        let name   = self.name;
        let args   = self.args.iter().enumerate().map(|(i, e)| {
            match *e {
                ArgDef::Type(ref t)        |
                ArgDef::TypeBound(ref t,_) => {
                    let n = syn::Ident::from(format!("a{}", i));
                    quote!(#n: #t)
                },
                ArgDef::SelfRefMut  => quote!(&mut self),
                ArgDef::SelfRef     => quote!(&self),
                ArgDef::SelfMove    => quote!(self),
                ArgDef::DisableSelf => {
                    panic!("got disable self late! (impossible)")
                },
            }
        });
        let bounds = self.args
            .iter()
            .filter(|e| e.is_type_bound())
            .map(|e| {
                if let ArgDef::TypeBound(_, ref tp) = *e {
                    tp.clone()
                } else {
                    panic!() /* already filtered out */
                }
        });

        let ret_ty = self.ret_ty.as_ref()
            .map_or(quote!(()), |ref rt| rt.into_tokens());

        let cfunc  = self.cfunc;
        let re = Regex::new("a[0-9]+").unwrap();
        let mut used_cargs = HashSet::new();
        self.cargs.iter().for_each(|e| {
            if let CArgDef::Immediate(ref expr) = *e {
                let mut ts = Tokens::new();
                expr.to_tokens(&mut ts);
                for m in re.find_iter(&ts.to_string()) {
                    used_cargs.insert(m.as_str().to_string());
                }
            }
        });

        let cargs: Vec<syn::Expr>  = self.cargs
            .iter().flat_map(|e| -> Vec<syn::Expr> {

            let ee = e.clone();
            match ee {
                CArgDef::Immediate(expr) => vec![expr],
                CArgDef::Rest(raw) => {
                        let mut r = vec![];
                        let all: Vec<String> = self.args
                            .iter()
                            .enumerate()
                            .filter(|&(_, e)| !e.is_self())
                            .map(|(i,_)| format!("a{}", i)).collect();
                        for a in all.iter()
                            .filter(|&e| !used_cargs.contains(e)) {

                            if raw {
                                r.push(syn::parse_str(a).expect("Parsing raw in carg"));
                            } else {
                                r.push(syn::parse_str(
                                        &format!("{}.data", a)
                                        ).expect("Parsing .data in carg"))
                            }
                        }
                        r
                }
            }
        }).collect();

        let mut body = quote!(
            #cfunc(#(#cargs),*)
        );

        body = if self.ret_from_raw {
            quote!(unsafe { Scm::_from_raw(#body) })
        } else {
            quote!(unsafe { #body })
        };

        if let syn::Type::Tuple(t) =
            syn::parse_str(&ret_ty.to_string()).expect("Parsing return type") {

            if t.elems.is_empty() {
                body = quote!(#body;);
            }
        }

        let _self = if self.self_disabled {
            quote!()
        } else {
            quote!(&self,)
        };

        tokens = quote!(
                #(#attrs)*
                #tokens fn #name<#(#bounds),*>
                    (#_self #(#args),*) -> #ret_ty {
                        #body
                    }
        );

        tokens
    }
}


named!(pub parse_guile_defs -> Vec<GuileDef>, do_parse!(
        content: call!(
            Punctuated::<GuileDef, Semi>::parse_terminated
            ) >>
        input_end!() >>
        (content.into_iter().collect())
));

named!(pub parse_guile_impl -> TokenStream, do_parse!(
        // TODO: imitate syn::ItemImpl with generics and things
        impld: syn!(syn::Type) >>
        body: map!(braces!(many0!(alt!(
                    tuple!(syn!(GuileDef), option!(punct!(;)))
                        => {
                            |(gd, _)| gd.construct()
                                         .into_tokens()
                                         .into()
                        }
                    |
                    syn!(syn::ImplItem)
                        => { |i| i.into_tokens().into() }

                    ))), |(_, c): (_, Vec<TokenStream>)| c) >>
        (quote!( impl #impld {
                    #(#body)*
                }).into())

));

