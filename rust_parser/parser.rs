#![deny(unused_must_use)]

use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use syn::parse_file;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};

pub struct RustImports {
    pub hints: Hints,
    pub imports: Vec<String>,
    pub test_imports: Vec<String>,
    pub extern_mods: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Hints {
    pub has_main: bool,
    pub has_test: bool,
    pub has_proc_macro: bool,
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
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    parse_imports_from_str(&contents)
}

pub fn parse_imports_from_str(contents: &str) -> Result<RustImports, Box<dyn Error>> {
    let ast = parse_file(contents)?;
    let mut visitor = AstVisitor::default();
    visitor.visit_file(&ast);

    let mut root_scope = visitor.mod_stack.pop_back().expect("no root scope");
    assert!(visitor.mod_stack.is_empty(), "leftover scopes");

    root_scope.trim_early_imports();

    let import_set: HashSet<_> = root_scope.imports.iter().collect();

    root_scope
        .test_imports
        .retain(|test_import| !import_set.contains(test_import));

    Ok(RustImports {
        hints: visitor.hints,
        imports: filter_imports(root_scope.imports),
        test_imports: filter_imports(root_scope.test_imports),
        extern_mods: visitor.extern_mods,
    })
}

fn filter_imports(imports: Vec<Ident>) -> Vec<String> {
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
    mods: HashSet<Ident<'ast>>,
    /// whether this scope is behind #[test] or #[cfg(test)]
    is_test_only: bool,
    /// whether this scope is behind #[gazelle::ignore]
    // TODO: this is not currently used, but we could support #[gazelle::ignore] on things like
    // functions and blocks in the future
    is_ignored: bool,
    /// crates that are imported in this scope
    imports: Vec<Ident<'ast>>,
    /// crates that are imported in test-only configurations in this scope
    test_imports: Vec<Ident<'ast>>,
}

impl<'ast> Scope<'ast> {
    /// Remove imports for mods that entered scope after the import appeared. This is uncommon, but
    /// it's possible to access an identifier that's used later in the same or a parent scope, or to
    /// access a module declared later in the file.
    fn trim_early_imports(&mut self) {
        self.imports.retain(|import| !self.mods.contains(import));
        self.test_imports
            .retain(|test_import| !self.mods.contains(test_import));
    }
}

#[derive(Debug)]
struct AstVisitor<'ast> {
    /// stack of mods in scope
    mod_stack: VecDeque<Scope<'ast>>,
    /// all mods that are currently in scope (including parent scopes)
    scope_mods: HashSet<Ident<'ast>>,
    /// collected hints
    hints: Hints,
    /// bare mods defined in external files
    extern_mods: Vec<String>,
    /// mods that are disallowed from being added to the current scope; this is currently only used
    /// for a hack, see below
    mod_denylist: HashSet<Ident<'ast>>,
}

impl<'ast> Default for AstVisitor<'ast> {
    fn default() -> Self {
        let mut mod_stack = VecDeque::new();
        mod_stack.push_back(Scope::default());
        Self {
            mod_stack,
            scope_mods: HashSet::default(),
            hints: Hints::default(),
            extern_mods: Vec::default(),
            mod_denylist: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Directive {
    Ignore,
}

impl<'ast> Directive {
    fn parse(meta: &'ast syn::Meta) -> Self {
        let path = meta.path();
        // TODO: proper error handling
        assert_eq!(
            path.segments.len(),
            2,
            "invalid gazelle directive: {:?}",
            path
        );
        assert_eq!(path.segments[0].ident, "gazelle");
        let ident = &path.segments[1].ident;

        // can't use match because we can't construct Idents to match against
        if ident == "ignore" {
            Self::Ignore
        } else {
            panic!("unexpected gazelle directive: {}", ident);
        }
    }
}

#[derive(Default)]
struct DirectiveSet {
    directives: HashSet<Directive>,
}

impl DirectiveSet {
    fn insert(&mut self, directive: Directive) {
        self.directives.insert(directive);
    }

    fn should_ignore(&self) -> bool {
        self.directives.contains(&Directive::Ignore)
    }
}

impl<'ast> AstVisitor<'ast> {
    fn add_import<I: Into<Ident<'ast>>>(&mut self, ident: I) {
        let ident = ident.into();

        if ident == "crate" || ident == "super" || ident == "self" {
            // these are keywords referring to the current crate; not an import
            return;
        }

        if !self.scope_mods.contains(&ident) && !self.is_ignored_scope() {
            if self.is_test_only_scope() {
                self.mod_stack.back_mut().unwrap().test_imports.push(ident);
            } else {
                self.mod_stack.back_mut().unwrap().imports.push(ident);
            }
        }
    }

    fn add_mod<I: Into<Ident<'ast>>>(&mut self, ident: I) {
        let ident = ident.into();

        if !self.scope_mods.contains(&ident) && !self.mod_denylist.contains(&ident) {
            self.scope_mods.insert(ident.clone());
            self.mod_stack.back_mut().unwrap().mods.insert(ident);
        }
    }

    fn push_scope(&mut self, test: bool, ignored: bool) {
        // TODO: create stack entry lazily so that we avoid it if there are no renames in this scope
        let current_scope = self.mod_stack.back().unwrap();
        self.mod_stack.push_back(Scope {
            mods: HashSet::new(),
            // scopes within test-only scopes are also test-only
            is_test_only: test || current_scope.is_test_only,
            is_ignored: ignored || current_scope.is_ignored,
            imports: vec![],
            test_imports: vec![],
        });
    }

    fn pop_scope(&mut self) {
        let mut scope = self.mod_stack.pop_back().expect("hit bottom of stack");

        for rename in &scope.mods {
            self.scope_mods.remove(rename);
        }

        scope.trim_early_imports();

        let parent_scope = self.mod_stack.back_mut().expect("no parent scope");

        parent_scope.imports.extend(scope.imports);
        parent_scope.test_imports.extend(scope.test_imports);
    }

    fn is_root_scope(&self) -> bool {
        self.mod_stack.len() == 1
    }

    fn is_test_only_scope(&self) -> bool {
        self.mod_stack.back().unwrap().is_test_only
    }

    fn is_ignored_scope(&self) -> bool {
        self.mod_stack.back().unwrap().is_ignored
    }

    fn visit_attr_meta(&mut self, meta: &syn::Meta) {
        // parse #[derive(A, B, ...)] and #[cfg_attr(..., ...)]
        match meta {
            syn::Meta::Path(path) => {
                if path.segments.len() > 1 {
                    self.add_import(
                        path.segments
                            .pairs()
                            .next()
                            .unwrap()
                            .into_value()
                            .ident
                            .clone(),
                    );
                }
            }
            syn::Meta::List(list) => {
                if let Some(ident) = list.path.get_ident() {
                    if ident == "derive" {
                        if let Ok(nested) = list.parse_args_with(
                            Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                        ) {
                            for derive in nested {
                                self.visit_attr_meta(&derive);
                            }
                        }
                    } else if ident == "cfg_attr" {
                        if let Ok(nested) = list.parse_args_with(
                            Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                        ) {
                            // grab second child, which is the inner attribute
                            let mut iter = nested.into_iter();
                            iter.next();
                            if let Some(inner) = iter.next() {
                                self.visit_attr_meta(&inner);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn parse_directives(&self, attrs: &'ast Vec<syn::Attribute>) -> DirectiveSet {
        let mut directives = DirectiveSet::default();
        for attr in attrs {
            if let syn::Meta::Path(path) = &attr.meta {
                if path
                    .segments
                    .first()
                    .map(|seg| seg.ident == "gazelle")
                    .unwrap_or(false)
                {
                    directives.insert(Directive::parse(&attr.meta));
                }
            }
        }
        directives
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
        let directives = self.parse_directives(&node.attrs);

        let mut imports = HashSet::new();

        // NOTE: We want to ignore any dependencies inside the ignored scope. However, we still want
        // to bring anything imported into scope, hence the visit::visit_item_use outside the
        // conditional below.
        if !directives.should_ignore() {
            parse_use_imports(&node.tree, &mut imports);
        }

        for import in &imports {
            self.add_import(import.clone());
        }

        // Name-only uses, e.g. `use foobar;`, don't bring anything new into scope that isn't
        // already in scope. This also applies to uses which bring their own import into scope,
        // e.g. `use foobar::foobar;`. If we were to permit this identifier to enter scope, our
        // logic would remove it from the imports list at the end of the scope, which is wrong.
        // This denylist approach is admittedly a big hack for lack of a better approach.
        self.mod_denylist = imports;
        visit::visit_item_use(self, node);
        self.mod_denylist.clear();
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
        let directives = self.parse_directives(&node.attrs);
        if !directives.should_ignore() {
            self.add_import(&node.ident);
        }
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.push_scope(false, false);
        visit::visit_block(self, node);
        self.pop_scope();
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let mut is_test_only = false;

        // parse #[cfg(test)]
        for attr in &node.attrs {
            if let syn::Meta::List(list) = &attr.meta {
                if let Some(ident) = list.path.get_ident() {
                    if ident == "cfg" {
                        if let Ok(nested) = attr.parse_args_with(
                            Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                        ) {
                            if nested.len() == 1 {
                                if let syn::Meta::Path(path) = &nested[0] {
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
            }
        }

        if self.is_root_scope() && node.content.is_none() {
            // this mod is defined in a different file
            self.extern_mods.push(node.ident.to_string());
        }

        self.add_mod(&node.ident);
        self.push_scope(is_test_only, false);
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
                if let syn::Meta::Path(path) = &attr.meta {
                    if let Some(ident) = path.get_ident() {
                        if ident == "test" {
                            self.hints.has_test = true;
                            is_test_only = true;
                        } else if ident == "proc_macro" || ident == "proc_macro_attribute" {
                            self.hints.has_proc_macro = true;
                        }
                    }
                }
            }
        }

        self.push_scope(is_test_only, false);
        visit::visit_item_fn(self, node);
        self.pop_scope();
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        visit::visit_item_enum(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        visit::visit_item_type(self, node);
    }

    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        self.visit_attr_meta(&node.meta);
        visit::visit_attribute(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if let Some(macro_ident) = node.mac.path.get_ident() {
            if macro_ident == "macro_rules" {
                if let Some(new_ident) = &node.ident {
                    self.add_mod(new_ident);
                }
            }
        }
        visit::visit_item_macro(self, node);
    }
}

fn parse_use_imports<'ast>(use_tree: &'ast syn::UseTree, imports: &mut HashSet<Ident<'ast>>) {
    match use_tree {
        syn::UseTree::Path(path) => {
            imports.insert(Ident::Ref(&path.ident));
        }
        syn::UseTree::Name(name) => {
            imports.insert(Ident::Ref(&name.ident));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                parse_use_imports(item, imports);
            }
        }
        _ => (),
    }
}
