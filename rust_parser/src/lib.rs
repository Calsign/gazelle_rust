#![deny(unused_must_use)]

use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use syn::parse_file;
use syn::visit::{self, Visit};

pub fn parse_imports(file: PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    // TODO: stream from the file instead of loading it all into memory
    let mut file = File::open(file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let ast = parse_file(&content)?;
    let mut visitor = AstVisitor::default();
    visitor.visit_file(&ast);

    let imports = visitor
        .imports
        .iter()
        .filter_map(|ident| {
            let s = ident.to_string();
            // uppercase is structs
            // TODO: don't store all the structs! seems wasteful
            if s.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                Some(s)
            } else {
                None
            }
        })
        .collect();

    Ok(imports)
}

#[derive(Debug)]
struct AstVisitor<'ast> {
    imports: HashSet<&'ast syn::Ident>,
    mod_stack: VecDeque<Vec<&'ast syn::Ident>>,
    scope_mods: HashSet<&'ast syn::Ident>,
}

impl<'ast> Default for AstVisitor<'ast> {
    fn default() -> Self {
        let mut mod_stack = VecDeque::new();
        mod_stack.push_back(Vec::new());
        Self {
            imports: HashSet::default(),
            mod_stack,
            scope_mods: HashSet::default(),
        }
    }
}

impl<'ast> AstVisitor<'ast> {
    fn add_import(&mut self, ident: &'ast syn::Ident) {
        if !self.scope_mods.contains(ident) {
            self.imports.insert(ident);
        }
    }

    fn add_mod(&mut self, ident: &'ast syn::Ident) {
        if !self.scope_mods.contains(ident) {
            self.scope_mods.insert(ident);
            self.mod_stack.back_mut().unwrap().push(ident);
        }
    }

    fn push_scope(&mut self) {
        // TODO: create stack entry lazily so that we avoid it if there are no renames in this scope
        self.mod_stack.push_back(Vec::new());
    }

    fn pop_scope(&mut self) {
        for rename in self.mod_stack.pop_back().expect("hit bottom of stack") {
            self.scope_mods.remove(rename);
        }
    }
}

impl<'ast> Visit<'ast> for AstVisitor<'ast> {
    fn visit_use_name(&mut self, node: &'ast syn::UseName) {
        self.add_mod(&node.ident);
    }

    fn visit_use_rename(&mut self, node: &'ast syn::UseRename) {
        self.add_import(&node.ident);
        self.add_mod(&node.rename);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if node.segments.len() > 1 {
            self.add_import(&node.segments[0].ident);
        }
        visit::visit_path(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        // the first path segment is an import
        match &node.tree {
            syn::UseTree::Path(path) => self.add_import(&path.ident),
            syn::UseTree::Name(name) => self.add_import(&name.ident),
            _ => (),
        }
        visit::visit_item_use(self, node);
    }

    fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
        // search for `x::y::{self, z}` because this puts `y` in scope
        if let syn::UseTree::Group(group) = &*node.tree {
            for tree in &group.items {
                match tree {
                    // TODO: not sure how else to test for "self" besides to_string
                    syn::UseTree::Name(name) if name.ident.to_string() == "self" => {
                        self.add_mod(&node.ident);
                    }
                    _ => (),
                }
            }
        }
        visit::visit_use_path(self, node);
    }

    fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate) {
        self.add_import(&node.ident);
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.push_scope();
        visit::visit_block(self, node);
        self.pop_scope();
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.add_mod(&node.ident);
        self.push_scope();
        visit::visit_item_mod(self, node);
        self.pop_scope();
    }
}
