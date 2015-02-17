#![feature(quote)]

#[macro_use]
extern crate log;

extern crate bindgen;
extern crate capnpc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate syntax;
extern crate toml;

mod config;

use bindgen::{Logger, Bindings, BindgenOptions};
use std::{fs, os};
use std::default::Default;
use std::old_io::{self, Writer};

use syntax::ast;
use syntax::parse;
use syntax::codemap::{ExpnInfo, NameAndSpan, MacroBang, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::ext::expand::ExpansionConfig;
use syntax::ext::quote::rt::ToSource;
use syntax::ptr::P;

struct StdLogger;
impl Logger for StdLogger {
    fn error(&self, msg: &str) {
        error!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        warn!("{}", msg);
    }
}

struct StubCtx<'a, 'b: 'a> {
    cx: &'a mut ExtCtxt<'b>,
    config: &'a config::Config,
    name: &'a str,
    argtys: &'a [&'a str],
    argnames: &'a [&'a str],
    ret: &'a str,
}

/*
struct StubCtx {
    cx: &mut ExtCtxt,
    config: &config::Config,
    name: &str,
    args: &[(&str, &str)],
    ret: &str,
}
*/

use syntax::parse::new_parser_from_source_str as npfss;

impl<'a, 'b> StubCtx<'a, 'b> {
    fn emit_log(&self, prefix: &str, body: &mut Vec<P<ast::Stmt>>) {
        let stmt = format!("writeln!(&mut proxy.log.borrow_mut(), \"{}{}\");", prefix, self.name);
        let mut parser = npfss(self.cx.parse_sess, self.cx.cfg(), "".to_string(), stmt);
        body.push(parser.parse_stmt(vec!()));
    }

    fn emit_passthru(&self, _: &mut Vec<P<ast::Stmt>>) -> P<ast::Expr> {
        let sig = format!("extern \"C\" fn({}) -> {}", self.argtys.connect(", "), self.ret);
        let expr = format!("unsafe {{
            static mut orig: Option<{0}> = None;
            if orig.is_none() {{
                let sym = rtld_next(\"{1}\");
                if sym == ptr::null_mut() {{
                    panic!(\"dlsym returned NULL for \\\"{1}\\\"\")
                }} else {{
                    orig = Some(mem::transmute(sym));
                }}
            }}
            (orig.unwrap())({2})
        }}", sig, self.name, self.argnames.connect(", "));
        let mut parser = npfss(self.cx.parse_sess, self.cx.cfg(), "".to_string(), expr);
        parser.parse_expr()
    }

    fn emit_logger(&self, body: &mut Vec<P<ast::Stmt>>) -> P<ast::Expr> {
        self.emit_log("", body);
        self.emit_passthru(body)
    }

    fn emit_proxy(&self, body: &mut Vec<P<ast::Stmt>>) -> P<ast::Expr> {
        self.emit_log("PROXYING ", body);

        let req = format!("let request = proxy.mb.{}_request();", self.name);
        let mut parser = npfss(self.cx.parse_sess, self.cx.cfg(), "".to_string(), req);
        body.push(parser.parse_stmt(vec!()));

        for &n in self.argnames {
            let stmt = format!("request.set_{0}({0});", n.to_string());
            let mut parser = npfss(self.cx.parse_sess, self.cx.cfg(), "".to_string(), stmt);
            body.push(parser.parse_stmt(vec!()));
        }

        body.push(quote_stmt!(self.cx, request.send().wait()));

        self.emit_passthru(body)
    }

    fn emit<W: old_io::Writer>(&self, out: &mut W) {
        let mut body = vec!();

        let expr = if self.config.proxy.contains(self.name) {
            self.emit_proxy(&mut body)
        } else {
            self.emit_logger(&mut body)
        };

        let args = self.argnames.iter().zip(self.argtys.iter()).map(|(&n, &t)| {
            let mut parser = npfss(self.cx.parse_sess, self.cx.cfg(), "".to_string(), t.to_string());
            let id = self.cx.ident_of(n);
            let ty = parser.parse_ty();
            self.cx.arg(DUMMY_SP, id, ty)
        });

        let block = self.cx.block(DUMMY_SP, body, Some(expr));
        let with_proxy = quote_expr!(self.cx, with_proxy(|proxy| $block));
        let mut parser = npfss(self.cx.parse_sess, self.cx.cfg(), "".to_string(), self.ret.to_string());
        let ret_ty = parser.parse_ty();
        let func = self.cx.item_fn(DUMMY_SP, self.cx.ident_of(self.name), args.collect(),
                                   ret_ty, self.cx.block(DUMMY_SP, vec!(), Some(with_proxy)));
        let func = P(ast::Item { vis: ast::Public, ..(*func).clone() });
        
        out.write_str("#[no_mangle]\n#[no_stack_check]\n#[linkage = \"external\"]\n");
        out.write_str(&func.to_source());
        out.write_str("\n\n");
    }
}

fn main() {
    let mut options: BindgenOptions = BindgenOptions {
        clang_args: [
            "-I/usr/include/glib-2.0",
            "-I/usr/lib/glib-2.0/include",
            "-I/usr/lib/clang/3.5.1/include",
            "-I/usr/include/libpurple",
            "/usr/include/libpurple/purple.h",
        ].iter().map(|&s| String::from_str(s)).collect(),
        .. Default::default()
    };

    let logger = StdLogger;

    options.types_only = true;
    match Bindings::generate(&options, Some(&logger as &Logger), None) {
        Ok(bindings) => {
            let out_dir = Path::new(os::getenv("OUT_DIR").unwrap());
            let mut out = old_io::fs::File::create(&out_dir.join("purple.rs")).unwrap();

            if let Err(e) = bindings.write(&mut out) {
                panic!("Unable to write bindings to file. {}", e);
            }
        }
        Err(_) => panic!()
    }

    let sess = parse::new_parse_sess();
    let cfg = ExpansionConfig {
        crate_name: "stubs".to_string(),
        enable_quotes: true,
        recursion_limit: 64,
    };
    
    let mut cx = ExtCtxt::new(&sess, vec!(), cfg);
    cx.bt_push(ExpnInfo {
        call_site: DUMMY_SP,
        callee: NameAndSpan {
            name: String::new(),
            format: MacroBang,
            span: None
        }
    });

    let config = match fs::File::open("stubs.toml") {
        Ok(file) => match config::parse_config(file) {
            Ok(config) => config,
            Err(e) => panic!("Config file error: {:?}", e),
        },
        Err(e) => panic!("error: {}", e),
    };

    options.types_only = false;
    options.sym_pat = vec!("^purple_".to_string());
    match Bindings::generate(&options, Some(&logger as &Logger), None) {
        Ok(bindings) => {
            let out_dir = Path::new(os::getenv("OUT_DIR").unwrap());
            let mut out = old_io::fs::File::create(&out_dir.join("stubs.rs")).unwrap();

            let items = bindings.into_ast();
            const FNS_IDX: usize = 1;
            if let ast::ItemForeignMod(ref fm) = items[FNS_IDX].node {
                for fi in &fm.items {
                    let name = fi.ident.to_source();
                    let ref item = fi.node;
                    if let ast::ForeignItemFn(ref sig, _) = *item {
                        let argnames: Vec<_> = sig.inputs.iter().map(|a| a.pat.to_source()).collect();
                        let argtys: Vec<_> = sig.inputs.iter().map(|a| a.ty.to_source()).collect();
                        let argnames: Vec<_> = argnames.iter().map(|s| s.as_slice()).collect();
                        let argtys: Vec<_> = argtys.iter().map(|s| s.as_slice()).collect();
                        let ret = match sig.output {
                            ast::NoReturn(_) => "!".to_string(),
                            ast::DefaultReturn(_) => "()".to_string(),
                            ast::Return(ref ty) => ty.to_source(),
                        };

                        let stub = StubCtx {
                            cx: &mut cx,
                            config: &config,
                            name: &name,
                            argnames: &argnames,
                            argtys: &argtys,
                            ret: &ret,
                        };

                        stub.emit(&mut out)
                    } else {
                        panic!();
                    }
                }
            }
        }
        Err(_) => panic!()
    }

    ::capnpc::compile(Path::new("."),
                      &[Path::new("proxy.capnp"),
                        Path::new("types.capnp")]);
}
