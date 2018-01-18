#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate regex;

extern crate proc_macro2;

use std::path::Path;
use std::fs::File;
use std::fs;
use std::env;
use std::io::{Read, Write};
use std::collections::HashSet;

use syn::visit_mut;
use syn::token::{Semi, Comma};
use syn::synom::{SynomBuffer, Synom};
use syn::punctuated::Punctuated;
use quote::{ToTokens, Tokens};
use proc_macro2::{TokenStream};

use regex::Regex;


enum ArgDef {
    Type(syn::Type),
    // TypeBound(syn::Type, syn::Ident, Punctuated<syn::TypeParamBound, Token![+]>),
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
            ArgDef::SelfRef | ArgDef::SelfRefMut | ArgDef::SelfMove => true,
            _ => false,
        }
    }
}

impl Synom for ArgDef {
    named!(parse -> Self, alt!(
            tuple!(punct!(@), punct!(_)) => { |_| ArgDef::DisableSelf }
            |
            tuple!(punct!(&), keyword!(mut), keyword!(self)) => {|_| ArgDef::SelfRefMut }
            |
            tuple!(punct!(&), keyword!(self)) => {|_| ArgDef::SelfRef }
            |
            tuple!(keyword!(self)) => {|_| ArgDef::SelfMove }
            |
            // syn!(syn::ArgSelfRef) => { |s| if s.mutability.is_some() { ArgDef::SelfRefMut } else { ArgDef::SelfRef } }
            // |
            // syn!(syn::ArgSelf) => { |_| ArgDef::SelfMove }
            do_parse!(
                type_: syn!(syn::Type) >>
                punct!(|) >>
                tparam: syn!(syn::TypeParam) >>
                (ArgDef::TypeBound(type_, tparam)))
            |
            syn!(syn::Type) => { |type_| ArgDef::Type(type_) }
    ));
}

#[derive(Debug)]
#[derive(Clone)]
enum CArgDef {
    Immediate(syn::Expr),
    Rest(bool),
}


impl Synom for CArgDef {
    named!(parse -> Self, alt!(
            syn!(syn::Expr) => { |e| CArgDef::Immediate(e) }
            |
            tuple!(punct!(!), punct!(*), option!(punct!(#))) => { |(_, _, r)| { CArgDef::Rest(r.is_none()) }}
    ));
}

fn expand_carg_flags<T: Synom>(tts: TokenStream) -> T {
    let out = tts.to_string()
        .replace("@ s", "self.data")
        .replace("@*", "! *");
    let re = Regex::new(r"@ ([0-9]+) #").unwrap();
    let out = re.replace_all(&out, |c: &regex::Captures| { format!("a{}.data", &c[1]) })
        .to_string()
        .replace("@ ", "a");
    eprintln!("EXPANDED: `{}`", out);
    syn::parse_str(&out).unwrap()
}

#[derive(Debug)]
struct CArgs { pub args: Vec<CArgDef> }
impl Synom for CArgs {
    named!(parse -> Self, do_parse!(
            args: map!(call!(Punctuated::<CArgDef, Comma>::parse_terminated),
                 |es| es.into_iter().collect::<Vec<CArgDef>>()) >>
            (Self {args: args})
    ));
}

struct GuileDef {
    pub public: bool,
    pub name: syn::Ident,
    pub self_disabled: bool,
    pub args: Vec<ArgDef>,
    pub cfunc: syn::Ident,
    pub cargs: Vec<CArgDef>,
    // pub cargs: Vec<syn::Expr>,
    pub ret_from_raw: bool,
    pub ret_ty: Option<syn::Type>,
}

impl Synom for GuileDef {
    named!(parse -> Self, do_parse!(
            public: alt!(keyword!(pub) => {|_| true}
                         |
                         epsilon!()    => {|_| false}) >>
            keyword!(fn) >>
            name: syn!(syn::Ident) >>
            args: map!(parens!(call!(Punctuated::<ArgDef, Comma>::parse_terminated)),
                        |paren| -> Vec<ArgDef> {
                            // paren.1.into_iter().map(|e| e.into_item()).collect()
                            paren.1.into_iter().collect()
                        }) >>

            cfunc_call: do_parse!(
                       punct!(=>) >>
                       cfunc: syn!(syn::Ident) >>
                       cargs: map!(parens!(syn!(TokenStream)), |p| expand_carg_flags::<CArgs>(p.1).args) >>
                       (cfunc, cargs)) >>

            ret_info: map!(option!(do_parse!(
                       punct!(->) >>
                       ir: map!(option!(tuple!(punct!(@), syn!(syn::Ident))),
                                |o| o.is_some() && o.unwrap().1.as_ref() == "r") >>
                       rt: syn!(syn::Type) >>
                       (ir, Some(rt)))),

                       |ri| if ri.is_none() { (false, None) } else { ri.unwrap() })>>

            (Self {
                public:        public,
                name:          name,
                self_disabled: args.iter().any(|a| a.is_disable_self() || a.is_self()),
                args:          args.into_iter().filter(|a| !a.is_disable_self()).collect(),
                cfunc:         cfunc_call.0,
                cargs:         cfunc_call.1,

                ret_from_raw: ret_info.0,
                ret_ty:       ret_info.1,
            })
    ));
}

impl GuileDef {
    pub fn construct(&self) -> quote::Tokens {
        let mut tokens = if self.public {quote!(pub)} else {quote::Tokens::new()};

        let name   = self.name;
        let args   = self.args.iter().enumerate().map(|(i, e)| {
            match *e {
                ArgDef::Type(ref t)        |
                ArgDef::TypeBound(ref t,_) => {
                    let n = syn::Ident::from(format!("a{}", i));
                    quote!(#n: #t)
                },
                ArgDef::SelfRefMut         => quote!(&mut self),
                ArgDef::SelfRef            => quote!(&self),
                ArgDef::SelfMove           => quote!(self),
                ArgDef::DisableSelf        => panic!("got disable self late! (impossible)"),
            }
        });
        let bounds = self.args.iter().filter(|e| e.is_type_bound()).map(|e| {
            if let ArgDef::TypeBound(_, ref tp) = *e { tp.clone() }
            else { panic!() /* already filtered out */ }
        });

        let ret_ty = self.ret_ty.as_ref().map_or(quote!(()), |ref rt| rt.into_tokens());
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
        let cargs: Vec<syn::Expr>  = self.cargs.iter().flat_map(|e| -> Vec<syn::Expr> {
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
                        for a in all.iter().filter(|&e| !used_cargs.contains(e)) {
                            if raw {
                                r.push(syn::parse_str(a).unwrap());
                            } else {
                                r.push(syn::parse_str(&format!("{}.data", a)).unwrap())
                            }
                        }
                        r
                }
            }
        }).collect();

        let mut body = quote!(
            unsafe { #cfunc(#(#cargs),*) }
        );

        if self.ret_from_raw {
            body = quote!(Scm::_from_raw(#body));
        }

        if let syn::Type::Tuple(t) = syn::parse_str(&ret_ty.to_string()).unwrap() {
            if t.elems.is_empty() {
                body = quote!(#body;);
            }
        }

        let _self = if self.self_disabled { quote!() } else { quote!(&self,) };
        tokens = quote!(
                #tokens fn #name<#(#bounds),*>(#_self #(#args),*) -> #ret_ty {
                    #body
                }
        );

        tokens
    }
}


named!(parse_guile_defs -> Vec<GuileDef>, do_parse!(
        content: call!(Punctuated::<GuileDef, Semi>::parse_terminated) >>
        input_end!() >>
        (content.into_iter().collect())
));

named!(parse_guile_impl -> TokenStream, do_parse!(
        // TODO: imitate syn::ItemImpl with generics and things
        impld: syn!(syn::Type) >>
        body: map!(braces!(many0!(alt!(
                    tuple!(syn!(GuileDef), option!(punct!(;))) => { |(gd, _)| gd.construct().into_tokens().into() }
                    |
                    syn!(syn::ImplItem) => { |i| i.into_tokens().into() }
                    ))), |(_, c): (_, Vec<TokenStream>)| c) >>
        (quote!( impl #impld {
                    #(#body)*
                }).into())

));

struct MacroVisitor;
impl visit_mut::VisitorMut for MacroVisitor {
    fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
        if i.path == syn::Path::from("guile_impl") {
            let sb = SynomBuffer::new(i.tts.clone().into());
            i.tts = parse_guile_impl(sb.begin()).expect("Expanding guile_impl macro").0;


        } else if i.path == syn::Path::from("guile_defs") {
            let sb = SynomBuffer::new(i.tts.clone().into());
            let gdefs = parse_guile_defs(sb.begin()).expect("Expanding guile_defs macro").0;

            let mut mtokens = quote::Tokens::new();

            gdefs.iter().for_each(|gd| gd.construct().to_tokens(&mut mtokens));
            i.tts = mtokens.into();
        }
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // let out_dir      = env::var("OUT_DIR").unwrap();

    let macro_targets = vec![
        (Path::new(&manifest_dir).join("expanded/scm.mod.rs"), Path::new(&manifest_dir).join("src/scm/mod.rs")),
        // add more files with macros to expand here...
    ];

    for (from, to) in macro_targets {
        fs::copy(&from, &to).unwrap();
        expand_macros(&to);
        // add external macros to expands here...
    }
}

fn expand_macros(target: &Path) {
    let mut source = String::new();
    File::open(target).unwrap().read_to_string(&mut source).unwrap();
    let mut file: syn::File = syn::parse_file(&source).expect("Parsing source");

    visit_mut::visit_file_mut(&mut MacroVisitor{}, &mut file);

    let mut tokens = Tokens::new();
    file.to_tokens(&mut tokens);
    File::create(target).unwrap()
        .write_all(
            tokens
            .to_string()
            .replace("{ ", "{\n")
            .replace(" }", "\n}")
            .replace("( ", "(")
            .replace(" )", ")")
            .replace(" ; ", ";\n")
            .replace(" !", "!")
            .as_bytes()
        ).unwrap();

}

