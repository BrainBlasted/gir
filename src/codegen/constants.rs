use analysis::imports::Imports;
use env::Env;
use std::path::Path;
use file_saver;
use codegen::general::{self, cfg_condition, cfg_deprecated, version_condition,
                       version_condition_string};
use library;

pub fn generate(env: &Env, root_path: &Path, mod_rs: &mut Vec<String>) {
    let path = root_path.join("constants.rs");
    let mut imports = Imports::new(&env.library);

    if env.analysis.constants.is_empty() {
        return;
    }

    let sys_crate_name = env.main_sys_crate_name();
    imports.add(sys_crate_name, None);
    imports.add("std::ffi::CStr", None);

    file_saver::save_to_file(path, env.config.make_backup, |w| {
        try!(general::start_comments(w, &env.config));
        try!(general::uses(w, env, &imports));
        try!(writeln!(w));

        mod_rs.push("\nmod constants;".into());

        for constant in &env.analysis.constants {
            let type_ = env.type_(constant.typ);
            if let library::Type::Fundamental(library::Fundamental::Utf8) = *type_ {
                try!(cfg_deprecated(
                    w,
                    env,
                    constant.deprecated_version,
                    false,
                    0
                ));
                try!(cfg_condition(w, &constant.cfg_condition, false, 0));
                try!(version_condition(w, env, constant.version, false, 0));
                try!(writeln!(w, "lazy_static! {{"));
                try!(writeln!(
                    w,
                    "    pub static ref {name}: &'static str = \
                     unsafe{{CStr::from_ptr({sys_crate_name}::{c_id}).to_str().unwrap()}};",
                    sys_crate_name = sys_crate_name,
                    name = constant.name,
                    c_id = constant.glib_name
                ));
                try!(writeln!(w, "}}"));
                if let Some(cfg) = version_condition_string(env, constant.version, false, 0) {
                    mod_rs.push(cfg);
                }
                mod_rs.push(format!("pub use self::constants::{};", constant.name));
            }
        }

        Ok(())
    });
}
