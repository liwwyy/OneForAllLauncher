fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");

        let version = env!("CARGO_PKG_VERSION");
        res.set("CompanyName", "OneForAllLauncher contributors");
        res.set("ProductName", "OneForAllLauncher");
        res.set("FileDescription", "OneForAllLauncher");
        res.set(
            "LegalCopyright",
            "© 2026 OneForAllLauncher contributors; original work © Polyfrost Inc.",
        );
        res.set("OriginalFilename", "oneforall_app.exe");
        res.set("InternalName", "oneforall_app");
        res.set("FileVersion", version);
        res.set("ProductVersion", version);

        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
