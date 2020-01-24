use super::{AstAttrValue, AstField, ErrorList, FullItem};
use rbdl_syn::RBDLFile;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use syn::{Error, Ident, Result};

#[derive(Debug)]
pub struct AstFile {
    pub items: HashMap<Ident, AstItem>,
}

impl TryFrom<RBDLFile> for AstFile {
    type Error = Error;
    fn try_from(parse_tree: RBDLFile) -> Result<AstFile> {
        let mut set = HashSet::new();
        let mut err = ErrorList::new();
        let mut items = HashMap::with_capacity(parse_tree.items.len());
        for item in parse_tree.items {
            match FullItem::try_from(item) {
                Err(e) => err.push_err(e),
                Ok(full_item) => {
                    let (ident, ast_item) = full_item.unwrap();
                    if set.contains(&ident) {
                        err.push(ident.span(), format!("`{}` is already defined before.", &ident));
                    } else {
                        set.insert(ident.clone());
                        items.insert(ident, ast_item);
                    }
                }
            }
        }
        match err.collapse() {
            Some(e) => Err(e),
            None => Ok(AstFile { items }),
        }
    }
}

#[derive(Debug)]
pub enum AstItem {
    Struct(AstItemContent),
    Enum(AstItemContent),
}

impl AstItem {
    pub fn unwrap(self) -> AstItemContent {
        match self {
            AstItem::Struct(s) => s,
            AstItem::Enum(e) => e,
        }
    }
    pub fn unwrap_ref(&self) -> &AstItemContent {
        match self {
            AstItem::Struct(s) => s,
            AstItem::Enum(e) => e,
        }
    }
    pub fn is_struct(&self) -> bool {
        match self {
            AstItem::Struct(_) => true,
            AstItem::Enum(_) => false,
        }
    }
    pub fn is_enum(&self) -> bool {
        match self {
            AstItem::Struct(_) => false,
            AstItem::Enum(_) => true,
        }
    }
}

#[derive(Debug)]
pub struct AstItemContent {
    pub attrs: HashMap<Ident, AstAttrValue>,
    pub fields: Vec<AstField>,
}

#[cfg(test)]
mod test_item {
    use super::*;
    use syn::parse_str;
    #[test]
    fn test_duplicate() {
        let parse_tree: RBDLFile = parse_str(
            "\
        x: struct{ \
            #[a, a=x, a = y]
            x: A\
        }\
        ",
        )
        .unwrap();
        let ast = AstFile::try_from(parse_tree);
        assert!(ast.is_err());
    }
}
