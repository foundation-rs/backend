fn main() {
    println!("cargo:rustc-link-search=/opt/oracle/instantclient_19_3/");    
    println!("cargo:rustc-link-lib=clntsh");
    println!("cargo:rustc-link-lib=nnz19");
    println!("cargo:rustc-link-lib=mql1");
    println!("cargo:rustc-link-lib=ipc1");
    println!("cargo:rustc-link-lib=clntshcore");
}