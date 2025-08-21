use std::{fmt::Display, path::Path};

/// A MIME type
pub enum ContentType {
        Text(ContentTypeText),
        Application(ContentTypeApplication),
        Image(ContentTypeImage),
}
/// A `text/` MIME type
pub enum ContentTypeText {
        Html,
        Css,
        Csv,
        Markdown,
        Plain,
}
/// An `application/` MIME type
pub enum ContentTypeApplication {
        Javascript,
        Json,
        Xml,
        Rtf,
}
/// An `image/` MIME type
pub enum ContentTypeImage {
        Png,
        Jpeg,
        Gif,
        Svg,
        Webp,
        Ico,
}

impl ContentType {
        /// Guesses the MIME type of a path based on its extension.
        /// Returns `None` if no MIME type can be guessed.
        pub fn from_endpoint<P: AsRef<Path>>(path: P) -> Option<Self> {
                fn inner(path: &Path) -> Option<ContentType> {
                        let extension = path.extension()?.to_string_lossy();
                        match extension.as_ref() {
                                "html" | "htm" => Some(ContentType::Text(ContentTypeText::Html)),
                                "css" => Some(ContentType::Text(ContentTypeText::Css)),
                                "csv" => Some(ContentType::Text(ContentTypeText::Csv)),
                                "md" => Some(ContentType::Text(ContentTypeText::Markdown)),
                                "txt" | "log" | "ini" | "cfg" | "conf" | "env" | "sh" | "bash" => {
                                        Some(ContentType::Text(ContentTypeText::Plain))
                                }
                                "js" => Some(ContentType::Application(ContentTypeApplication::Javascript)),
                                "json" => Some(ContentType::Application(ContentTypeApplication::Json)),
                                "xml" => Some(ContentType::Application(ContentTypeApplication::Xml)),
                                "rtf" => Some(ContentType::Application(ContentTypeApplication::Rtf)),
                                "png" => Some(ContentType::Image(ContentTypeImage::Png)),
                                "jpeg" | "jpg" => Some(ContentType::Image(ContentTypeImage::Jpeg)),
                                "gif" => Some(ContentType::Image(ContentTypeImage::Gif)),
                                "svg" => Some(ContentType::Image(ContentTypeImage::Svg)),
                                "webp" => Some(ContentType::Image(ContentTypeImage::Webp)),
                                "ico" => Some(ContentType::Image(ContentTypeImage::Ico)),
                                _ => None,
                        }
                }
                inner(path.as_ref())
        }
}

impl Display for ContentType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let (ty, subtype) = match self {
                        Self::Text(ty) => {
                                let prefix = "text";
                                let suffix = match ty {
                                        ContentTypeText::Html => "html",
                                        ContentTypeText::Css => "css",
                                        ContentTypeText::Csv => "csv",
                                        ContentTypeText::Markdown => "markdown",
                                        ContentTypeText::Plain => "plain",
                                };
                                (prefix, suffix)
                        }
                        Self::Application(ty) => {
                                let prefix = "application";
                                let suffix = match ty {
                                        ContentTypeApplication::Javascript => "javascript",
                                        ContentTypeApplication::Json => "json",
                                        ContentTypeApplication::Xml => "xml",
                                        ContentTypeApplication::Rtf => "rtf",
                                };
                                (prefix, suffix)
                        }
                        Self::Image(ty) => {
                                let prefix = "image";
                                let suffix = match ty {
                                        ContentTypeImage::Png => "png",
                                        ContentTypeImage::Jpeg => "jpeg",
                                        ContentTypeImage::Gif => "gif",
                                        ContentTypeImage::Svg => "svg+xml",
                                        ContentTypeImage::Webp => "webp",
                                        ContentTypeImage::Ico => "x-icon",
                                };
                                (prefix, suffix)
                        }
                };
                write!(f, "{ty}/{subtype}")
        }
}
