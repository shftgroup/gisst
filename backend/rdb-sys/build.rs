fn main() {
    const LDB_DIR: &str = "libretro-util/libretro-db";
    const LC_DIR: &str = "libretro-util/libretro-common";
    let files = vec![
        format!("{LDB_DIR}/rmsgpack.c"),
        format!("{LDB_DIR}/rmsgpack_dom.c"),
        format!("{LDB_DIR}/libretrodb_tool.c"),
        format!("{LDB_DIR}/bintree.c"),
        format!("{LDB_DIR}/query.c"),
        format!("{LDB_DIR}/libretrodb.c"),
        format!("{LC_DIR}/compat/compat_fnmatch.c"),
        format!("{LC_DIR}/string/stdstring.c"),
        format!("{LC_DIR}/streams/file_stream.c"),
        format!("{LC_DIR}/compat/compat_strcasestr.c"),
        format!("{LC_DIR}/time/rtime.c"),
        format!("{LC_DIR}/file/file_path.c"),
        format!("{LC_DIR}/file/file_path_io.c"),
        format!("{LC_DIR}/vfs/vfs_implementation.c"),
        format!("{LC_DIR}/encodings/encoding_utf.c"),
        format!("{LC_DIR}/compat/compat_strl.c"),
        format!("{LC_DIR}/compat/fopen_utf8.c"),
    ];
    let includes = vec![LDB_DIR.to_string(), format!("{LC_DIR}/include")];
    cc::Build::new()
        .includes(includes)
        .files(files)
        .static_flag(true)
        .warnings(false)
        .flag("-g")
        .define("NDEBUG", None)
        .flag("-O0")
        .compile("retro-db");
}
