use std::error::Error;

use libloading::{Library, Symbol};

use finance_together_api::capi::cbindings::{pluginMain, Handler};

pub struct Loader {
    libs: Vec<Library>,
}

impl Loader {

    pub fn new() -> Loader {
        Loader { libs: Vec::new() }
    }

    pub fn load_library(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let lib = unsafe { Library::new(filename)? };
        unsafe { lib.get::<Symbol<pluginMain>>(b"pluginMain")?(handlerRegister)}; 
        self.libs.push(lib);

        Ok(())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn handlerRegister(handler: Handler) -> bool {
    true
}