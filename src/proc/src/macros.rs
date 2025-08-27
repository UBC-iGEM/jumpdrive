use helpers::content_types::ContentType;
use ignore::WalkBuilder;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{
        collections::HashSet,
        env,
        fmt::Write as _,
        io,
        path::{Path, PathBuf},
        process::{Command, Stdio},
};
use syn::{
        Error, Ident, LitStr, braced,
        parse::Parse,
        parse_macro_input,
        punctuated::Punctuated,
        token::{Colon, Comma, Eq},
};

mod keywords {
        syn::custom_keyword!(dir);
        syn::custom_keyword!(ws);
        syn::custom_keyword!(routes);
        syn::custom_keyword!(err);
}

struct MacroInput {
        map_dir: LitStr,
        socket: Option<(LitStr, Ident)>,
        other_paths: Punctuated<PathItem, Comma>,
        error_handler: Ident,
}

impl Parse for MacroInput {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let _: keywords::dir = input.parse()?;
                let _: Eq = input.parse()?;
                let map_dir: LitStr = input.parse()?;
                let _: Comma = input.parse()?;

                let socket = if input.peek(keywords::ws) {
                        let _: keywords::ws = input.parse()?;
                        let _: Eq = input.parse()?;

                        let socket: LitStr = input.parse()?;
                        let _: Colon = input.parse()?;
                        let socket_closure: Ident = input.parse()?;

                        let _: Comma = input.parse()?;
                        Some((socket, socket_closure))
                } else {
                        None
                };

                let other_paths = if input.peek(keywords::routes) {
                        let _: keywords::routes = input.parse()?;
                        let _: Eq = input.parse()?;
                        let content;
                        braced!(content in input);
                        let routes = content.parse_terminated(PathItem::parse, Comma)?;
                        let _: Comma = input.parse()?;
                        routes
                } else {
                        Punctuated::new()
                };

                let _: keywords::err = input.parse()?;
                let _: Eq = input.parse()?;
                let error_handler: Ident = input.parse()?;

                if input.peek(Comma) {
                        let _: Comma = input.parse()?;
                }

                if input.is_empty() {
                        let output = MacroInput {
                                map_dir,
                                socket,
                                other_paths,
                                error_handler,
                        };
                        Ok(output)
                } else {
                        Err(Error::new(input.span(), "Trailing tokens detected..."))
                }
        }
}

struct PathItem(LitStr, Ident);
impl Parse for PathItem {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let path: LitStr = input.parse()?;
                let _: Colon = input.parse()?;
                let callback: Ident = input.parse()?;
                Ok(Self(path, callback))
        }
}

/// The primary entrypoint for Jumpdrive.
/// ## Example:
/// ```rust
/// jumpdrive! {
///         dir = "public/",
///         ws = "/ws": websocket_handler,
///         routes = {
///                 "/csv": csv_server
///         },
///         err = error_handler
/// }
/// ```
/// ## Required fields:
/// - **dir**: the path of a directory relative to `CARGO_MANIFEST_DIR`
/// - **err**: a callback function which executes upon encountering errors
/// ## Optional fields:
/// - **ws**:
///     - an endpoint to serve Websocket connections, and
///     - a callback function which executes when a new Websocket connection is established
/// - **routes**: a list of
///     - endpoints, and
///     - callback functions which execute when a `GET` request is made to their endpoint
#[proc_macro]
pub fn jumpdrive(input: TokenStream) -> TokenStream {
        let macro_input = parse_macro_input!(input as MacroInput);
        let (mut path_map, (stripped_paths, absolute_paths)) = match get_paths(macro_input.map_dir) {
                Ok(v) => v,
                Err(e) => return e.into_compile_error().into(),
        };
        let mime_type = match stripped_paths
                .iter()
                .map(|v| -> Result<String, TokenStream> {
                        match ContentType::from_endpoint(v.value()) {
                                Some(content_type) => Ok(content_type.to_string()),
                                None => Err(
                                        syn::Error::new(v.span(), format!("Could not resolve content type of file {}", v.value()))
                                                .into_compile_error()
                                                .into(),
                                ),
                        }
                })
                .collect::<Result<Vec<_>, _>>()
        {
                Ok(paths) => paths,
                Err(e) => return e,
        };

        if let Some((ref socket_path, _)) = macro_input.socket
                && let Err(e) = handle_additional_path(socket_path, &mut path_map)
        {
                return e.into_compile_error().into();
        }
        let socket_arg = match macro_input.socket {
                Some((socket_path, socket_handler)) => quote!(Some((#socket_path, #socket_handler))),
                None => quote!(None),
        };

        if let Err(e) = macro_input
                .other_paths
                .iter()
                .try_for_each(|PathItem(p_lit, _)| handle_additional_path(p_lit, &mut path_map))
        {
                return e.into_compile_error().into();
        }

        let (path_arg, path_handler): (Vec<_>, Vec<_>) = macro_input
                .other_paths
                .into_iter()
                .map(|PathItem(arg, handler)| (arg, handler))
                .collect();

        let error_handler = macro_input.error_handler;

        quote! {
            ::jumpdrive::Jumpdrive::new(
                ::jumpdrive::prelude::phf_map! {
                    #(#stripped_paths => (include_bytes!(#absolute_paths), #mime_type)),*
                },
                #socket_arg,
                ::jumpdrive::prelude::phf_map! {
                    #(#path_arg => #path_handler),*
                },
                #error_handler,
            ).serve()
        }
        .into()
}

type ServePaths = (HashSet<PathBuf>, (Vec<LitStr>, Vec<LitStr>));
fn get_paths(target: LitStr) -> Result<ServePaths, syn::Error> {
        let span = target.span();
        let target = Path::new(
                &env::var("CARGO_MANIFEST_DIR")
                        .map_err(|e| syn::Error::new(span, format!("Failed to determine crate root with err {e:?}")))?,
        )
        .join(Path::new(&target.value()));
        if !target.exists() {
                return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Requested directory {target:?} does not exist!"),
                ));
        }

        if cfg!(feature = "tsc") {
                // let tsc_glob = format!("{}**.ts", target.display());
                match Command::new("tsc")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .current_dir(&target)
                        .output()
                {
                        Err(e) => return Err(syn::Error::new(span, format!("Failed to spawn tsc with err {e:?}"))),
                        Ok(output) => {
                                if !output.status.success() {
                                        let mut err_msg = String::from("Tsc failed!\n");
                                        if !output.stdout.is_empty() {
                                                let _ = write!(err_msg, "Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
                                        }
                                        if !output.stderr.is_empty() {
                                                let _ = write!(err_msg, "Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
                                        }
                                        return Err(syn::Error::new(span, err_msg));
                                }
                        }
                }
        }

        let mut path_pairs = Vec::new();
        let walk = WalkBuilder::new(&target)
                .add_custom_ignore_filename(".jumpdriveignore")
                .ignore(true)
                .git_global(false)
                .git_exclude(false)
                .git_ignore(false)
                .build();
        walk.into_iter()
                .try_for_each(|f| -> Result<(), io::Error> {
                        let entry = f.map_err(|e| io::Error::other(format!("Failed to parse ignore file! {e:?}")))?;
                        let abs_path = entry.path();
                        // Skip TS files and directories
                        if let Some(ext) = abs_path.extension()
                                && ext == "ts"
                        {
                                return Ok(());
                        }
                        if abs_path.is_dir() {
                                return Ok(());
                        }
                        let stripped_path = abs_path.strip_prefix(&target).map_err(|e| {
                                io::Error::new(
                                        io::ErrorKind::InvalidFilename,
                                        format!("Prefix stripping failed with err {e}. This is a logical error!"),
                                )
                        })?;
                        path_pairs.push((stripped_path.to_path_buf(), abs_path.to_path_buf()));
                        Ok(())
                })
                .map_err(|e| syn::Error::new(span, e))?;

        let (stripped_paths, absolute_paths) = path_pairs
                .iter()
                .map(|(stripped_path, absolute_path)| {
                        (
                                LitStr::new(&stripped_path.to_string_lossy(), span),
                                LitStr::new(&absolute_path.to_string_lossy(), span),
                        )
                })
                .collect();
        let path_set: HashSet<_> = path_pairs.into_iter().map(|(p, _)| p).collect();
        Ok((path_set, (stripped_paths, absolute_paths)))
}

fn handle_additional_path(path_lit: &LitStr, other_paths: &mut HashSet<PathBuf>) -> Result<(), syn::Error> {
        let path_str = path_lit.value();
        if let Some('/') = path_str.chars().next() {
                // Splice the leading '/'
                let path = PathBuf::from(&path_str[1..]);
                if !other_paths.insert(path) {
                        Err(syn::Error::new(
                                path_lit.span(),
                                format!("Multiple definitions of path '{}'!", path_lit.value()),
                        ))
                } else {
                        Ok(())
                }
        } else {
                Err(syn::Error::new(
                        path_lit.span(),
                        format!("Path '{path_str}' is not prefixed with a '/'!"),
                ))
        }
}
