use proc_macro::{Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use std::{
    collections::HashSet,
    env,
    fmt::Display,
    io,
    ops::Not,
    path::{Path, PathBuf},
};
use syn::{
    Error, Ident, LitStr, braced, bracketed,
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

            let content;
            bracketed!(content in input);
            let socket: LitStr = content.parse()?;
            let _: Comma = content.parse()?;
            let socket_closure: Ident = content.parse()?;

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

#[proc_macro]
#[proc_macro_error]
pub fn jumpdrive(input: TokenStream) -> TokenStream {
    let macro_input = parse_macro_input!(input as MacroInput);
    let (mut path_map, (stripped_paths, absolute_paths)) = serve_paths(macro_input.map_dir);
    let mime_type: Vec<_> = stripped_paths
        .iter()
        .map(|v| {
            ContentType::new(v)
                .unwrap_or_else(|| {
                    abort!(
                        v.span(),
                        format!("Could not resolve content type of file {}", v.value())
                    )
                })
                .to_string()
        })
        .collect();

    if let Some((ref socket_path, _)) = macro_input.socket {
        handle_additional_path(socket_path, &mut path_map);
    }
    let socket_arg = match macro_input.socket {
        Some((socket_path, socket_handler)) => quote!(Some((#socket_path, #socket_handler))),
        None => quote!(None),
    };

    macro_input
        .other_paths
        .iter()
        .for_each(|PathItem(p_lit, _)| {
            handle_additional_path(p_lit, &mut path_map);
        });

    let (path_arg, path_handler): (Vec<_>, Vec<_>) = macro_input
        .other_paths
        .into_iter()
        .map(|PathItem(arg, handler)| (arg, handler))
        .collect();

    let error_handler = macro_input.error_handler;

    quote! {
        ::jumpdrive::Jumpdrive {
            map: ::phf::phf_map! {
                #(#stripped_paths => (include_bytes!(#absolute_paths), #mime_type)),*
            },
            socket: #socket_arg,
            other_paths: ::phf::phf_map! {
                #(#path_arg => #path_handler),*
            },
            error_handler: #error_handler,
        }.serve()
    }
    .into()
}

fn recursive_read(
    dir: &Path,
    original_path: &Path,
    path_pairs: &mut Vec<(PathBuf, PathBuf)>,
) -> io::Result<()> {
    for entry in dir.read_dir()? {
        let abs_path = entry?.path();
        let stripped_path = abs_path.strip_prefix(original_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidFilename,
                format!("Prefix stripping failed with err {e}. This is a logical error!"),
            )
        })?;
        if abs_path.is_dir() {
            recursive_read(&abs_path, original_path, path_pairs)?;
        } else {
            path_pairs.push((stripped_path.to_path_buf(), abs_path));
        }
    }
    Ok(())
}

fn serve_paths(target: LitStr) -> (HashSet<PathBuf>, (Vec<LitStr>, Vec<LitStr>)) {
    let span = target.span();
    // Canonicalize parent directory of current crate
    let crate_home = Path::new(
        &env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|e| abort!(span, format!("Failed to determine crate root: {e}"))),
    )
    .join(Path::new(&target.value()));
    if crate_home.exists().not() {
        abort!(
            Span::call_site(),
            format!("Requested directory {crate_home:?} does not exist!")
        )
    }

    let mut path_map = Vec::new();
    recursive_read(&crate_home, &crate_home, &mut path_map).unwrap_or_else(|e| {
        abort!(
            Span::call_site(),
            format!("Failed to read {crate_home:?}: {e}")
        )
    });

    let (stripped_paths, absolute_paths) = path_map
        .iter()
        .map(|(stripped_path, absolute_path)| {
            (
                LitStr::new(&stripped_path.to_string_lossy(), span),
                LitStr::new(&absolute_path.to_string_lossy(), span),
            )
        })
        .collect();
    let path_set: HashSet<_> = path_map.iter().map(|(p, _)| p.clone()).collect();
    (path_set, (stripped_paths, absolute_paths))
}

fn handle_additional_path(path_lit: &LitStr, other_paths: &mut HashSet<PathBuf>) {
    let path_str = path_lit.value();
    if let Some('/') = path_str.chars().next() {
        // Splice the leading '/'
        let path = PathBuf::from(&path_str[1..]);
        if !other_paths.insert(path) {
            abort!(
                path_lit.span(),
                format!("Multiple definitions of path '{}'!", path_lit.value())
            )
        }
    } else {
        abort!(
            path_lit.span(),
            format!("Path '{path_str}' is not prefixed with a '/'!"),
        )
    }
}

enum ContentType {
    Text(TextContentType),
    Application(ApplicationContentType),
    Image(ImageContentType),
}
enum TextContentType {
    Html,
    Css,
    Markdown,
    Plain,
}
enum ApplicationContentType {
    Javascript,
    Json,
    Xml,
    Rtf,
}
enum ImageContentType {
    Png,
    Jpeg,
    Gif,
    Svg,
    Webp,
    Ico,
}

impl ContentType {
    fn new(path: &LitStr) -> Option<ContentType> {
        let path = &path.value()[1..];
        let extension = Path::new(&path).extension()?.to_string_lossy();
        match extension.as_ref() {
            "html" | "htm" => Some(Self::Text(TextContentType::Html)),
            "css" => Some(Self::Text(TextContentType::Css)),
            "md" => Some(Self::Text(TextContentType::Markdown)),
            "txt" | "log" | "csv" | "ini" | "cfg" | "conf" | "env" | "sh" | "bash" => {
                Some(Self::Text(TextContentType::Plain))
            }
            "js" => Some(Self::Application(ApplicationContentType::Javascript)),
            "json" => Some(Self::Application(ApplicationContentType::Json)),
            "xml" => Some(Self::Application(ApplicationContentType::Xml)),
            "rtf" => Some(Self::Application(ApplicationContentType::Rtf)),
            "png" => Some(Self::Image(ImageContentType::Png)),
            "jpeg" | "jpg" => Some(Self::Image(ImageContentType::Jpeg)),
            "gif" => Some(Self::Image(ImageContentType::Gif)),
            "svg" => Some(Self::Image(ImageContentType::Svg)),
            "webp" => Some(Self::Image(ImageContentType::Webp)),
            "ico" => Some(Self::Image(ImageContentType::Ico)),
            _ => None,
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (ty, subtype) = match self {
            Self::Text(ty) => {
                let prefix = "text";
                let suffix = match ty {
                    TextContentType::Html => "html",
                    TextContentType::Css => "css",
                    TextContentType::Markdown => "markdown",
                    TextContentType::Plain => "plain",
                };
                (prefix, suffix)
            }
            Self::Application(ty) => {
                let prefix = "application";
                let suffix = match ty {
                    ApplicationContentType::Javascript => "javascript",
                    ApplicationContentType::Json => "json",
                    ApplicationContentType::Xml => "xml",
                    ApplicationContentType::Rtf => "rtf",
                };
                (prefix, suffix)
            }
            Self::Image(ty) => {
                let prefix = "image";
                let suffix = match ty {
                    ImageContentType::Png => "png",
                    ImageContentType::Jpeg => "jpeg",
                    ImageContentType::Gif => "gif",
                    ImageContentType::Svg => "svg+xml",
                    ImageContentType::Webp => "webp",
                    ImageContentType::Ico => "x-icon",
                };
                (prefix, suffix)
            }
        };
        write!(f, "{ty}/{subtype}")
    }
}
