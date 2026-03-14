#![deny(unused_must_use)]

use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use proc_macro2::{TokenStream, TokenTree};
use syn::ext::IdentExt;
use syn::parse_file;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};

pub struct RustImports {
    pub hints: Hints,
    pub imports: Vec<String>,
    pub test_imports: Vec<String>,
    pub extern_mods: Vec<String>,
    pub compile_data: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Hints {
    pub has_main: bool,
    pub has_test: bool,
    pub has_proc_macro: bool,
}

pub fn parse_imports(
    absolute_path: PathBuf,
    relative_path: PathBuf,
    enabled_features: &[String],
) -> Result<RustImports, Box<dyn Error>> {
    // TODO: stream from the file instead of loading it all into memory?
    let mut file = match File::open(&absolute_path) {
        Err(err) => {
            eprintln!(
                "Could not open file {}: {}",
                absolute_path.to_str().unwrap_or("<utf-8 decode error>"),
                err,
            );
            std::process::exit(1);
        }
        file => file?,
    };
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    parse_imports_from_str(&contents, enabled_features, relative_path)
}

pub fn parse_imports_from_str(
    contents: &str,
    enabled_features: &[String],
    path: PathBuf,
) -> Result<RustImports, Box<dyn Error>> {
    let ast = parse_file(contents)?;
    let mut visitor = AstVisitor::new(enabled_features, path);
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
        extern_mods: visitor.extern_mods.into_iter().collect(),
        compile_data: visitor.compile_data.into_iter().collect(),
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

impl From<syn::Ident> for Ident<'_> {
    fn from(ident: syn::Ident) -> Self {
        Self::Owned(ident)
    }
}

impl PartialEq<&str> for Ident<'_> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::Ref(ident) => ident == other,
            Self::Owned(ident) => ident == other,
        }
    }
}

impl Ident<'_> {
    // NOTE: this is just matching the wrapping the implementation in syn::Ident
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        match self {
            Self::Ref(ident) => ident.unraw().to_string(),
            Self::Owned(ident) => ident.unraw().to_string(),
        }
    }

    fn into_owned<'a>(self) -> Ident<'a> {
        match self {
            Self::Ref(ident) => Ident::Owned(ident.clone()),
            Self::Owned(ident) => Ident::Owned(ident),
        }
    }
}

#[derive(Debug, Default, Clone)]
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

impl Scope<'_> {
    /// Remove imports for mods that entered scope after the import appeared. This is uncommon, but
    /// it's possible to access an identifier that's used later in the same or a parent scope, or to
    /// access a module declared later in the file.
    fn trim_early_imports(&mut self) {
        self.imports.retain(|import| !self.mods.contains(import));
        self.test_imports
            .retain(|test_import| !self.mods.contains(test_import));
    }
}

#[derive(Debug, Clone)]
struct AstVisitor<'ast> {
    /// The relative path from the root of Bazel package to the directory that contains the file we
    /// are parsing. This is used to resolve location of files that are included with include_str!
    /// and include_bytes!.
    containing_dir: PathBuf,
    /// stack of mods in scope
    mod_stack: VecDeque<Scope<'ast>>,
    /// all mods that are currently in scope (including parent scopes)
    scope_mods: HashSet<Ident<'ast>>,
    /// collected hints
    hints: Hints,
    /// bare mods defined in external files
    extern_mods: HashSet<String>,
    /// mods that are disallowed from being added to the current scope; this is currently only used
    /// for a hack, see below
    mod_denylist: HashSet<Ident<'ast>>,
    /// Enabled features
    enabled_features: HashSet<String>,
    /// Files that are included via include_str! and include_bytes! macros.
    compile_data: HashSet<String>,
}

impl AstVisitor<'_> {
    fn new(enabled_features: &[String], path: PathBuf) -> Self {
        let mut mod_stack = VecDeque::new();
        mod_stack.push_back(Scope::default());

        let containing_dir = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

        Self {
            containing_dir,
            mod_stack,
            scope_mods: HashSet::default(),
            hints: Hints::default(),
            extern_mods: HashSet::new(),
            mod_denylist: HashSet::new(),
            enabled_features: enabled_features.iter().cloned().collect(),
            compile_data: HashSet::new(),
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
    fn cfg_enabled(&self, attrs: &[syn::Attribute]) -> bool {
        for attr in attrs {
            if let syn::Meta::List(list) = &attr.meta {
                if list.path.is_ident("cfg") {
                    if let Ok(meta) = attr.parse_args::<syn::Meta>() {
                        if !self.eval_cfg_meta(&meta) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn eval_cfg_meta(&self, meta: &syn::Meta) -> bool {
        match meta {
            syn::Meta::Path(path) => {
                if let Some(ident) = path.get_ident() {
                    ident == "test"
                } else {
                    true
                }
            }

            syn::Meta::NameValue(nv) => {
                if nv.path.is_ident("feature") {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(lit) = &expr_lit.lit {
                            return self.enabled_features.contains(lit.value().as_str());
                        }
                    }
                }
                true
            }

            syn::Meta::List(list) => {
                if list.path.is_ident("any") {
                    list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                        .map(|args| args.iter().any(|m| self.eval_cfg_meta(m)))
                        .unwrap_or(true)
                } else if list.path.is_ident("all") {
                    list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                        .map(|args| args.iter().all(|m| self.eval_cfg_meta(m)))
                        .unwrap_or(true)
                } else if list.path.is_ident("not") {
                    list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                        .map(|args| {
                            if args.len() == 1 {
                                !self.eval_cfg_meta(&args[0])
                            } else {
                                true
                            }
                        })
                        .unwrap_or(true)
                } else {
                    true
                }
            }
        }
    }

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

    fn visit_macro_tokens(&mut self, tokens: TokenStream) {
        if tokens.is_empty() {
            return;
        }

        // Try parsing as an expression
        if let Ok(expr) = syn::parse2::<syn::Expr>(tokens.clone()) {
            self.push_scope(false, false);
            let mut macro_visitor = self.clone();
            macro_visitor.visit_expr(&expr);
            self.copy_from_visitor(macro_visitor);
            self.pop_scope();
            return;
        }

        // Try parsing as a statement
        if let Ok(stmt) = syn::parse2::<syn::Stmt>(tokens.clone()) {
            self.push_scope(false, false);
            let mut macro_visitor = self.clone();
            macro_visitor.visit_stmt(&stmt);
            self.copy_from_visitor(macro_visitor);
            self.pop_scope();
            return;
        }

        // Still nothing, we need to use some heuristics in order to select a parseable fragment of
        // the token stream.
        if starts_with_ident(&tokens) {
            // The token stream starts with an identifier but does not parse; it must be a list of
            // arguments separated by commas. Split it on commas and try parsing each argument
            // individually. We should only retry if we actually encountered some commas, otherwise
            // we risk running in an infinite loop.
            let (fragments, seen_commas) = split_on_commas(tokens);
            if seen_commas {
                for fragment in fragments {
                    self.visit_macro_tokens(fragment);
                }
            }
        } else {
            // The token stream does not start with an identifier: the parseable part probably comes
            // later. Strip everything until we hit an identifier and try again.
            self.visit_macro_tokens(remove_prefix_until_ident(tokens))
        }
    }

    fn copy_from_visitor(&mut self, other: AstVisitor<'_>) {
        self.mod_stack = VecDeque::new();
        for scope in other.mod_stack {
            let scope_copy = Scope {
                mods: scope.mods.into_iter().map(Ident::into_owned).collect(),
                imports: scope.imports.into_iter().map(Ident::into_owned).collect(),
                test_imports: scope
                    .test_imports
                    .into_iter()
                    .map(Ident::into_owned)
                    .collect(),
                is_test_only: scope.is_test_only,
                is_ignored: scope.is_ignored,
            };
            self.mod_stack.push_back(scope_copy);
        }
        for id in other.scope_mods {
            let id_copy = match id {
                Ident::Ref(ident) => Ident::Owned((*ident).clone()),
                Ident::Owned(ident) => Ident::Owned(ident),
            };
            self.scope_mods.insert(id_copy);
        }
        for id in other.mod_denylist {
            let id_copy = match id {
                Ident::Ref(ident) => Ident::Owned((*ident).clone()),
                Ident::Owned(ident) => Ident::Owned(ident),
            };
            self.mod_denylist.insert(id_copy);
        }
        self.extern_mods = other.extern_mods;
        self.compile_data = other.compile_data;
        self.enabled_features = other.enabled_features;
        self.hints.has_main = other.hints.has_main;
        self.hints.has_test = other.hints.has_test;
        self.hints.has_proc_macro = other.hints.has_proc_macro;
    }
}

/// Returns true if the path represents a test attribute.
///
/// Recognizes:
/// - Standard: #[test]
/// - Async/custom test frameworks: any attribute ending in ::test
///   (e.g., #[tokio::test], #[async_std::test], #[custom::framework::test])
fn is_test_attribute(path: &syn::Path) -> bool {
    // Single segment: #[test]
    if let Some(ident) = path.get_ident() {
        return ident == "test";
    }

    // Multi-segment: check if last segment is "test"
    path.segments.last().is_some_and(|seg| seg.ident == "test")
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
        if !self.cfg_enabled(&node.attrs) {
            return;
        }

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
        if !self.cfg_enabled(&node.attrs) {
            return;
        }

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
        if !self.cfg_enabled(&node.attrs) {
            return;
        }

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
            self.extern_mods.insert(node.ident.unraw().to_string());
        }

        self.add_mod(&node.ident);
        self.push_scope(is_test_only, false);
        visit::visit_item_mod(self, node);
        self.pop_scope();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if !self.cfg_enabled(&node.attrs) {
            return;
        }

        let mut is_test_only = false;

        if self.is_root_scope() && node.sig.ident == "main" {
            // main function in the top-level scope
            self.hints.has_main = true;
        } else {
            for attr in &node.attrs {
                match &attr.meta {
                    syn::Meta::Path(path) | syn::Meta::List(syn::MetaList { path, .. }) => {
                        if is_test_attribute(path) {
                            self.hints.has_test = true;
                            is_test_only = true;
                        } else if let Some(ident) = path.get_ident() {
                            if ident == "proc_macro" || ident == "proc_macro_attribute" {
                                self.hints.has_proc_macro = true;
                            }
                        }
                    }
                    _ => {}
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
        if !self.cfg_enabled(&node.attrs) {
            return;
        }

        if let Some(macro_ident) = node.mac.path.get_ident() {
            if macro_ident == "macro_rules" {
                if let Some(new_ident) = &node.ident {
                    self.add_mod(new_ident);
                }
            }
        }
        visit::visit_item_macro(self, node);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let macro_ident = mac.path.get_ident();

        if let Some(ident) = macro_ident {
            if ident == "include_str" || ident == "include_bytes" {
                if self.is_ignored_scope() {
                    return;
                }

                if let Ok(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                })) = syn::parse2::<syn::Expr>(mac.tokens.clone())
                {
                    let included_path = PathBuf::from(lit.value());

                    if included_path.is_absolute() {
                        panic!(
                            "included paths must not be absolute: {}",
                            included_path.display()
                        )
                    } else {
                        let combined = self.containing_dir.join(included_path);
                        match normalize_path(&combined).to_str() {
                            None => panic!("Invalid unicode in the path: {}", combined.display()),
                            Some(x) => self.compile_data.insert(x.to_string()),
                        };
                    }
                }
            }
        }
        self.visit_macro_tokens(mac.tokens.clone());
        visit::visit_macro(self, mac);
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

/// Normalize a path by resolving `.` and `..` without touching the filesystem
fn normalize_path(path: &Path) -> PathBuf {
    let mut stack = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if let Some(last) = stack.last() {
                    // Only pop if last is a normal component, not RootDir
                    if matches!(last, std::path::Component::Normal(_)) {
                        stack.pop();
                    } else {
                        stack.push(component);
                    }
                } else {
                    stack.push(component);
                }
            }
            std::path::Component::CurDir => { /* skip `.` */ }
            _ => stack.push(component),
        }
    }

    stack.iter().collect()
}

/// Remove all tokens before the first `Ident`. Return the remaining `TokenStream` starting with
/// that `Ident`.
fn remove_prefix_until_ident(ts: TokenStream) -> TokenStream {
    ts.into_iter()
        .skip_while(|tt| !matches!(tt, TokenTree::Ident(_)))
        .collect()
}

/// Split a `TokenStream` on commas, returning:
/// - A `Vec` of `TokenStreams` (segments without the commas)
/// - A bool indicating whether any comma was seen
fn split_on_commas(ts: TokenStream) -> (Vec<TokenStream>, bool) {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut seen_comma = false;

    for tt in ts {
        match &tt {
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                seen_comma = true;
                // Push current segment and start a new one
                result.push(current.into_iter().collect());
                current = Vec::new();
            }
            _ => current.push(tt),
        }
    }

    // Push the last segment if any tokens remain
    if !current.is_empty() || !seen_comma {
        result.push(current.into_iter().collect());
    }

    (result, seen_comma)
}

/// Return true if the given `TokenStream` starts with an `Ident`.
fn starts_with_ident(ts: &TokenStream) -> bool {
    ts.clone()
        .into_iter()
        .next()
        .is_some_and(|tt| matches!(tt, TokenTree::Ident(_)))
}
