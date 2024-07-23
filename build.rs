use std::{
    env, 
    path::PathBuf, 
    // process::Command,
};

use include_idl::compress_idl;

fn main() {
    // Run shank to generate the IDL
    // let _output = Command::new("pnpm")
    //     .arg("generate:idls")
    //     .output()
    //     .expect("Failed to run shank");

    // Get the IDL path
    let idl_path = PathBuf::from("/home/honey/projects/mcpay/idl").join("mcpay_0.json");
    print!("idl_path {:?}\n", idl_path);
    
    // Concat output path of compressed IDL
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("idl.json.zip");
    print!("dest_path {:?}", dest_path);
    
    compress_idl(&idl_path, &dest_path);
}