// Helper macros

macro_rules! ident {
    ($input: expr) => {
        Ident::new($input, Span::call_site())
    };
}

macro_rules! append_path {
    ($path: expr, $ident: expr) => {
        {
            let mut path = $path.clone();
            path.segments.push(PathSegment { ident: $ident, arguments: PathArguments::None });
            path
        }
    };
}

// Replaces the last segment in a path with a new ident
macro_rules! change_path_ident {
    ($path: expr, $ident: expr) => {
        {
            let mut path = $path.clone();
            if let Some(x) = path.segments.last_mut() {
                x.ident = $ident;
            } else {
                path.segments.push(PathSegment { ident: $ident, arguments: PathArguments::None });
            }
            path
        }
    };
}
