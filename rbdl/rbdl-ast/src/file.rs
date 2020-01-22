use super::{AstAttrValue, AstField, FullItem};
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
        let mut err = None;
        let mut items = HashMap::with_capacity(parse_tree.items.len());
        for item in parse_tree.items {
            match FullItem::try_from(item) {
                Err(e) => {
                    if err.is_none() {
                        err = Some(e);
                    } else {
                        err.as_mut().unwrap().combine(e);
                    }
                }
                Ok(full_item) => {
                    let (ident, ast_item) = full_item.unwrap();
                    if set.contains(&ident) {
                        let e = Error::new(ident.span(), format!("`{}` is already defined before.", &ident));
                        if err.is_none() {
                            err = Some(e);
                        } else {
                            err.as_mut().unwrap().combine(e);
                        }
                    } else {
                        set.insert(ident.clone());
                        items.insert(ident, ast_item);
                    }
                }
            }
        }
        Ok(AstFile { items })
    }
}

#[derive(Debug)]
pub enum AstItem {
    Struct(AstItemContent),
    Enum(AstItemContent),
}

#[derive(Debug)]
pub struct AstItemContent {
    pub attrs: HashMap<Ident, AstAttrValue>,
    pub fields: Vec<AstField>,
}
