#![feature(libc,path,core,fs,io,linkage,std_misc)]
#![allow(dead_code)]

extern crate capnp;
extern crate "capnp-rpc" as capnp_rpc;
extern crate libc;

pub const RTLD_NEXT: *mut u8 = 0xffffffffffffffff as *mut u8;

pub mod stubs;

mod types_capnp {
    include! { concat!(env!("OUT_DIR"), "/types_capnp.rs") }
}

mod proxy_capnp {
    include! { concat!(env!("OUT_DIR"), "/proxy_capnp.rs") }
}

mod purpleproxy {
    use std::cell::RefCell;
    use std::fs::File;

    use libc;

    use capnp_rpc::ez_rpc::EzRpcClient;
    use proxy_capnp::mockingbird::Client as Mockingbird;

    pub struct Proxy {
        pub log: RefCell<File>,
        pub mb: Mockingbird,
        rpc: EzRpcClient,
    }

    impl Proxy {
        fn new() -> Proxy {
            let path = Path::new(format!("purple{}.log", unsafe{libc::getpid()}));
            let log = File::create(&path).unwrap_or_else(
                |e| unsafe {
                    println!("Error: Couldn't open `{}` for writing: {}", path.display(), e);
                    exit(1);
                });

            let mut rpc = EzRpcClient::new("127.0.0.1:1337").unwrap_or_else(
                |e| unsafe {
                    println!("Error: Couldn't start rpc client: {}", e);
                    exit(1);
                });

            let mb = rpc.import_cap::<Mockingbird>("mockingbird");

            Proxy {
                log: RefCell::new(log),
                mb: mb,
                rpc: rpc,
            }
        }
    }

    extern "C" {
        fn exit(status: u8) -> !;
    }

    #[no_stack_check]
    pub fn with_proxy<T, F>(f: F) -> T 
            where F: FnOnce(&Proxy) -> T {
        thread_local!(static PROXY: Proxy = Proxy::new());

        PROXY.with(|p| {
            f(p)
        })
    }
}

/*
extern "C" {
    fn _exit(status: u8) -> !;
}

#[no_mangle]
#[no_stack_check]
#[linkage = "external"]
pub unsafe extern "C" fn __libc_start_main(main: Option<extern "C" fn (int, 
                                                                       *const *const u8,
                                                                       *const *const u8) -> int>,
                                           argc: int,
                                           argv: *const *const u8,
                                           init: Option<extern "C" fn ()>,
                                           fini: Option<extern "C" fn ()>,
                                           rtld_fini: Option<extern "C" fn ()>,
                                           stack_end: *mut ::libc::c_void) -> int {
    use std::dynamic_lib::dl;

    // I'm so sorry :(
    static mut orig: 
        Option<extern "C" fn (Option<extern "C" fn (int,
                                                    *const *const u8,
                                                    *const *const u8) -> int>,
                              int,
                              *const *const u8,
                              Option<extern "C" fn ()>,
                              Option<extern "C" fn ()>,
                              Option<extern "C" fn ()>,
                              *mut ::libc::c_void) -> int> = None;
    if orig.is_none() {
        let sym = dl::symbol(RTLD_NEXT, "__libc_start_main".as_ptr() as *const i8);
        if sym == ::std::ptr::null_mut() {
            println!("dlsym returned NULL for \"__libc_start_main\"");
            _exit(1);
        } else {
            orig = Some(::std::mem::transmute(sym));
        }
    }

    native::start(argc, argv, move || {
        (orig.unwrap())(main, argc, argv, init, fini, rtld_fini, stack_end);
    })
}
*/
