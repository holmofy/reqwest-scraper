use syn::Type;

pub(crate) fn get_type_detail(ty: &Type) -> PathType {
    match ty {
        Type::Path(typepath) if typepath.qself.is_none() => {
            let idents_of_path = typepath
                .path
                .segments
                .iter()
                .map(|ps| ps.ident.to_string())
                .collect::<Vec<_>>()
                .join(":");

            if vec!["Option", "std:option:Option", "core:option:Option"]
                .into_iter()
                .find(|s| idents_of_path == *s)
                .and_then(|_| typepath.path.segments.last())
                .is_some()
            {
                return PathType::Option;
            }

            if vec!["Vec", "std::vec::Vec", "core::vec::Vec", "alloc::vec::Vec"]
                .into_iter()
                .find(|s| idents_of_path == *s)
                .and_then(|_| typepath.path.segments.last())
                .is_some()
            {
                return PathType::Vector;
            }

            PathType::Other
        }
        _ => PathType::Other,
    }
}

pub(crate) enum PathType {
    Option,
    Vector,
    Other,
}

impl PathType {
    pub fn is_other(&self) -> bool {
        matches!(self, Self::Other)
    }
}
