#![deny(unused_must_use)]

use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use syn::parse_file;
use syn::visit::{self, Visit};

use messages_rust_proto::Hints;

pub struct RustImports {
    pub hints: Hints,
    pub imports: Vec<String>,
    pub test_imports: Vec<String>,
}

pub fn parse_imports(path: PathBuf) -> Result<RustImports, Box<dyn Error>> {
    // TODO: stream from the file instead of loading it all into memory?
    let mut file = match File::open(&path) {
        Err(err) => {
            eprintln!(
                "Could not open file {}: {}",
                path.to_str().unwrap_or("<utf-8 decode error>"),
                err,
            );
            std::process::exit(1);
        }
        file => file?,
    };
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let ast = parse_file(&content)?;
    let mut visitor = AstVisitor::default();
    visitor.visit_file(&ast);

    let test_imports = visitor
        .test_imports
        .difference(&visitor.imports)
        .cloned()
        .collect();

    Ok(RustImports {
        hints: visitor.hints,
        imports: filter_imports(visitor.imports),
        test_imports: filter_imports(test_imports),
    })
}

fn filter_imports(imports: HashSet<Ident>) -> Vec<String> {
    imports
        .into_iter()
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
        .collect()
}

// Macros aren't parsed as part of the overall AST, so when we parse them we get an owned value.
// This approach allows us to store both the references and the owned values together, minimzing
// clones.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Ident<'ast> {
    Ref(&'ast syn::Ident),
    Owned(syn::Ident),
}

impl<'ast> From<&'ast syn::Ident> for Ident<'ast> {
    fn from(ident: &'ast syn::Ident) -> Self {
        Self::Ref(ident)
    }
}

impl<'ast> From<syn::Ident> for Ident<'ast> {
    fn from(ident: syn::Ident) -> Self {
        Self::Owned(ident)
    }
}

impl<'ast> PartialEq<&str> for Ident<'ast> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::Ref(ident) => ident == other,
            Self::Owned(ident) => ident == other,
        }
    }
}

impl<'ast> Ident<'ast> {
    // NOTE: this is just matching the wrapping the implementation in syn::Ident
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        match self {
            Self::Ref(ident) => ident.to_string(),
            Self::Owned(ident) => ident.to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct Scope<'ast> {
    /// mods in scope
    mods: Vec<Ident<'ast>>,
    /// whether this scope is behind #[test] or #[cfg(test)]
    is_test_only: bool,
}

#[derive(Debug)]
struct AstVisitor<'ast> {
    /// crates that are imported
    imports: HashSet<Ident<'ast>>,
    /// crates that are imported in test-only configurations
    test_imports: HashSet<Ident<'ast>>,
    /// stack of mods in scope
    mod_stack: VecDeque<Scope<'ast>>,
    /// all mods that are currently in scope (including parent scopes)
    scope_mods: HashSet<Ident<'ast>>,
    /// collected hints
    hints: Hints,
}

impl<'ast> Default for AstVisitor<'ast> {
    fn default() -> Self {
        let mut mod_stack = VecDeque::new();
        mod_stack.push_back(Scope::default());
        Self {
            imports: HashSet::default(),
            test_imports: HashSet::default(),
            mod_stack,
            scope_mods: HashSet::default(),
            hints: Hints::default(),
        }
    }
}

impl<'ast> AstVisitor<'ast> {
    fn add_import<I: Into<Ident<'ast>>>(&mut self, ident: I) {
        let ident = ident.into();

        if ident == "crate" {
            // "crate" is a keyword referring to the current crate; not an import
            return;
        }

        if !self.scope_mods.contains(&ident) {
            if self.is_test_only_scope() {
                self.test_imports.insert(ident);
            } else {
                self.imports.insert(ident);
            }
        }
    }

    fn add_mod<I: Into<Ident<'ast>>>(&mut self, ident: I) {
        let ident = ident.into();

        if !self.scope_mods.contains(&ident) {
            self.scope_mods.insert(ident.clone());
            self.mod_stack.back_mut().unwrap().mods.push(ident);
        }
    }

    fn push_scope(&mut self, test: bool) {
        // TODO: create stack entry lazily so that we avoid it if there are no renames in this scope
        let current_test_only = self.mod_stack.back().unwrap().is_test_only;
        self.mod_stack.push_back(Scope {
            mods: Vec::new(),
            // scopes within test-only scopes are also test-only
            is_test_only: test || current_test_only,
        });
    }

    fn pop_scope(&mut self) {
        for rename in self.mod_stack.pop_back().expect("hit bottom of stack").mods {
            self.scope_mods.remove(&rename);
        }
    }

    fn is_root_scope(&self) -> bool {
        self.mod_stack.len() == 1
    }

    fn is_test_only_scope(&self) -> bool {
        self.mod_stack.back().unwrap().is_test_only
    }

    fn visit_type_attrs(&mut self, attrs: &'ast Vec<syn::Attribute>) {
        // parse #[derive(A, B, ...)]
        for attr in attrs {
            if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
                if let Some(ident) = list.path.get_ident() {
                    if ident == "derive" {
                        for nested in list.nested {
                            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = nested {
                                if path.segments.len() > 1 {
                                    // this dance moves it out to avoid a clone
                                    self.add_import(
                                        path.segments
                                            .into_pairs()
                                            .next()
                                            .unwrap()
                                            .into_value()
                                            .ident,
                                    );
                                }
                            }
                        }
                    }
                }
            }
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
                    syn::UseTree::Name(name) if name.ident == "self" => {
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
        self.push_scope(false);
        visit::visit_block(self, node);
        self.pop_scope();
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let mut is_test_only = false;

        // parse #[cfg(test)]
        for attr in &node.attrs {
            if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
                if let Some(ident) = list.path.get_ident() {
                    if ident == "cfg" && list.nested.len() == 1 {
                        if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = &list.nested[0] {
                            if let Some(ident) = path.get_ident() {
                                if ident == "test" {
                                    is_test_only = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.add_mod(&node.ident);
        self.push_scope(is_test_only);
        visit::visit_item_mod(self, node);
        self.pop_scope();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let mut is_test_only = false;

        if self.is_root_scope() && node.sig.ident == "main" {
            // main function in the top-level scope
            self.hints.has_main = true;
        } else {
            for attr in &node.attrs {
                if let Ok(syn::Meta::Path(path)) = attr.parse_meta() {
                    if let Some(ident) = path.get_ident() {
                        if ident == "test" {
                            self.hints.has_test = true;
                            is_test_only = true;
                        } else if ident == "proc_macro" {
                            self.hints.has_proc_macro = true;
                        }
                    }
                }
            }
        }

        self.push_scope(is_test_only);
        visit::visit_item_fn(self, node);
        self.pop_scope();
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.visit_type_attrs(&node.attrs);
        visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.visit_type_attrs(&node.attrs);
        visit::visit_item_enum(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.visit_type_attrs(&node.attrs);
        visit::visit_item_type(self, node);
    }
}
