#![deny(unused_must_use)]

use std::borrow::{Borrow, Cow};
use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use boolean_expression::Expr as BExpr;
use syn::parse_file;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};

use messages_rust_proto::Hints;

pub struct RustImports {
    pub hints: Hints,
    pub imports: HashMap<String, ConfigFlag>,
    pub extern_mods: Vec<String>,
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

    Ok(RustImports {
        hints: visitor.hints,
        imports: visitor
            .imports
            .into_iter()
            .map(|(imp, cfg)| (imp.to_string(), cfg.simplify_via_bdd()))
            .collect(),
        extern_mods: visitor.extern_mods,
    })
}

// Macros aren't parsed as part of the overall AST, so when we parse them we get an owned value.
// This approach allows us to store both the references and the owned values together, minimizing
// clones.
pub type Ident<'ast> = Cow<'ast, syn::Ident>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BExprAtom {
    Option { option: String },
    KeyOption { key: String, value: String },
}

pub type ConfigFlag = BExpr<BExprAtom>;

pub fn bexpr_join<T, C, I>(constructor: C, iter: I) -> Option<BExpr<T>>
where
    T: std::fmt::Debug + Clone + Eq + std::hash::Hash,
    C: Fn(Box<BExpr<T>>, Box<BExpr<T>>) -> BExpr<T>,
    I: Iterator<Item = BExpr<T>>,
{
    iter.fold(None, |acc, val| {
        Some(match acc {
            None => val,
            Some(acc) => constructor(Box::new(acc), Box::new(val)),
        })
    })
}

fn parse_meta(meta: &syn::Meta) -> syn::Result<ConfigFlag> {
    Ok(match meta {
        syn::Meta::Path(path) => BExpr::Terminal(BExprAtom::Option {
            option: path.require_ident()?.to_string(),
        }),
        syn::Meta::NameValue(syn::MetaNameValue { path, value, .. }) => {
            BExpr::Terminal(BExprAtom::KeyOption {
                key: path.require_ident()?.to_string(),
                value: match value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => s.value(),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            value,
                            "key-value cfg attr expects string value",
                        ))
                    }
                },
            })
        }
        syn::Meta::List(list) => {
            let nested = list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )?;

            if list.path.is_ident("all") {
                bexpr_join(
                    BExpr::And,
                    nested
                        .iter()
                        .map(parse_meta)
                        .collect::<syn::Result<Vec<_>>>()?
                        .into_iter(),
                )
                .expect("cfg(all(..)) contains no predicates")
            } else if list.path.is_ident("any") {
                bexpr_join(
                    ConfigFlag::Or,
                    nested
                        .iter()
                        .map(parse_meta)
                        .collect::<syn::Result<Vec<_>>>()?
                        .into_iter(),
                )
                .expect("cfg(any(..)) contains no predicates")
            } else if list.path.is_ident("not") {
                if nested.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        list,
                        "'not' cfg expects exactly one child",
                    ));
                }
                let inner = nested.first().unwrap();

                BExpr::Not(Box::new(parse_meta(inner)?))
            } else {
                // TODO: are there valid cfg attrs that looks like this?
                eprintln!("list: {:#?}", list);
                return Err(syn::Error::new_spanned(list, "unexpected cfg"));
            }
        }
    })
}
#[derive(Debug)]
enum LazyConfigFlag<'ast> {
    Lazy {
        list: &'ast syn::MetaList,
        parsed: OnceCell<ConfigFlag>,
    },
    Raw(ConfigFlag),
}

impl<'ast> LazyConfigFlag<'ast> {
    fn parse(&self) -> &ConfigFlag {
        match self {
            Self::Lazy { list, parsed } => parsed.get_or_init(|| {
                let meta: syn::Meta = list
                    .parse_args()
                    .expect("cfg attr expects only one predicate");

                parse_meta(&meta).expect("failed to parse cfg attr")
            }),
            Self::Raw(config_flag) => config_flag,
        }
    }
}

#[derive(Debug, Default)]
struct Scope<'ast> {
    /// mods in scope
    mods: Vec<Ident<'ast>>,
    /// whether this scope is behind #[gazelle::ignore]
    // TODO: this is not currently used, but we could support #[gazelle::ignore] on things like
    // functions and blocks in the future
    is_ignored: bool,
    config_flags: Vec<LazyConfigFlag<'ast>>,
}

#[derive(Debug)]
struct AstVisitor<'ast> {
    /// crates that are imported
    imports: HashMap<Ident<'ast>, ConfigFlag>,
    /// stack of mods in scope
    mod_stack: Vec<Scope<'ast>>,
    /// all mods that are currently in scope (including parent scopes)
    scope_mods: HashSet<Ident<'ast>>,
    /// collected hints
    hints: Hints,
    /// bare mods defined in external files
    extern_mods: Vec<String>,
    /// scratch buffer used for examining the first character of idents
    scratch_buffer: String,
}

impl<'ast> Default for AstVisitor<'ast> {
    fn default() -> Self {
        Self {
            imports: HashMap::default(),
            mod_stack: vec![Scope::default()],
            scope_mods: HashSet::default(),
            hints: Hints::default(),
            extern_mods: Vec::default(),
            scratch_buffer: String::with_capacity(30),
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
    fn add_import(&mut self, ident: Ident<'ast>) {
        let borrowed: &syn::Ident = ident.borrow();

        // these are keywords referring to the current crate; not an import
        if ["crate", "super", "self"]
            .iter()
            .any(|keyword| borrowed == keyword)
        {
            return;
        }

        // Skip identifiers that begin with uppercase letters since these are structs, not modules.
        // By writing into a reusable buffer, we avoid allocating a new string for each check.
        self.scratch_buffer.clear();
        write!(&mut self.scratch_buffer, "{}", &ident).expect("failed to format ident");
        if self
            .scratch_buffer
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or_default()
        {
            return;
        }

        if !self.scope_mods.contains(&ident) && !self.is_ignored_scope() {
            use std::collections::hash_map::Entry;

            let flags = self.current_config_flags();
            match self.imports.entry(ident) {
                Entry::Occupied(occupied) => {
                    let (ident, existing) = occupied.remove_entry();
                    self.imports
                        .insert(ident, BExpr::Or(Box::new(existing), Box::new(flags)));
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(flags);
                }
            }
        }
    }

    fn add_mod(&mut self, ident: Ident<'ast>) {
        if !self.scope_mods.contains(&ident) {
            self.scope_mods.insert(ident.clone());
            self.mod_stack.last_mut().unwrap().mods.push(ident);
        }
    }

    fn current_scope(&self) -> &Scope<'ast> {
        self.mod_stack.last().unwrap()
    }

    fn current_config_flags(&self) -> ConfigFlag {
        bexpr_join(
            ConfigFlag::And,
            self.mod_stack
                .iter()
                .flat_map(|scope| &scope.config_flags)
                .map(|flag| flag.parse().clone()),
        )
        .unwrap_or(BExpr::Const(true))
    }

    fn push_scope(&mut self, ignored: bool, config_flags: Vec<LazyConfigFlag<'ast>>) {
        // TODO: create stack entry lazily so that we avoid it if there are no renames in this scope
        let current_scope = self.mod_stack.last().unwrap();
        self.mod_stack.push(Scope {
            mods: Vec::new(),
            // scopes within ignored scopes are also ignored
            is_ignored: ignored || current_scope.is_ignored,
            config_flags,
        });
    }

    fn pop_scope(&mut self) {
        for rename in self.mod_stack.pop().expect("hit bottom of stack").mods {
            self.scope_mods.remove(&rename);
        }
    }

    fn is_root_scope(&self) -> bool {
        self.mod_stack.len() == 1
    }

    fn is_ignored_scope(&self) -> bool {
        self.current_scope().is_ignored
    }

    fn visit_type_attrs(&mut self, attrs: &'ast [syn::Attribute]) {
        // parse #[derive(A, B, ...)]
        for attr in attrs {
            if let syn::Meta::List(list) = &attr.meta {
                if list.path.is_ident("derive") {
                    if let Ok(nested) = attr
                        .parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                    {
                        for derive in nested {
                            if let syn::Meta::Path(path) = derive {
                                if path.segments.len() > 1 {
                                    // this dance moves it out to avoid a clone
                                    self.add_import(Cow::Owned(
                                        path.segments
                                            .into_pairs()
                                            .next()
                                            .unwrap()
                                            .into_value()
                                            .ident,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn visit_cfg_attrs(&mut self, attrs: &'ast [syn::Attribute]) -> Vec<LazyConfigFlag<'ast>> {
        let mut cfg_flags = Vec::new();

        // look for #[cfg(...)]
        for attr in attrs {
            if let syn::Meta::List(list) = &attr.meta {
                if list.path.is_ident("cfg") {
                    cfg_flags.push(LazyConfigFlag::Lazy {
                        list,
                        parsed: OnceCell::new(),
                    });
                }
            }
        }

        // TODO: parse cfg_attr; this would only apply to the dependencies within the conditional
        // attrs themselves

        cfg_flags
    }

    fn parse_directives(&self, attrs: &'ast [syn::Attribute]) -> DirectiveSet {
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
        self.add_mod(Cow::Borrowed(&node.ident));
    }

    fn visit_use_rename(&mut self, node: &'ast syn::UseRename) {
        self.add_import(Cow::Borrowed(&node.ident));
        self.add_mod(Cow::Borrowed(&node.rename));
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if node.segments.len() > 1 {
            self.add_import(Cow::Borrowed(&node.segments[0].ident));
        }
        visit::visit_path(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let directives = self.parse_directives(&node.attrs);

        // NOTE: We want to ignore any dependencies inside the ignored scope. However, we still want
        // to bring anything imported into scope, hence the visit::visit_item_use outside the
        // conditional below.
        if !directives.should_ignore() {
            // the first path segment is an import
            match &node.tree {
                syn::UseTree::Path(path) => self.add_import(Cow::Borrowed(&path.ident)),
                syn::UseTree::Name(name) => self.add_import(Cow::Borrowed(&name.ident)),
                _ => (),
            }
        }

        visit::visit_item_use(self, node);
    }

    fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
        // search for `x::y::{self, z}` because this puts `y` in scope
        if let syn::UseTree::Group(group) = &*node.tree {
            for tree in &group.items {
                match tree {
                    syn::UseTree::Name(name) if name.ident == "self" => {
                        self.add_mod(Cow::Borrowed(&node.ident));
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
            self.add_import(Cow::Borrowed(&node.ident));
        }
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.push_scope(false, Vec::new());
        visit::visit_block(self, node);
        self.pop_scope();
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let cfg = self.visit_cfg_attrs(&node.attrs);

        if self.is_root_scope() && node.content.is_none() {
            // this mod is defined in a different file
            self.extern_mods.push(node.ident.to_string());
        }

        self.add_mod(Cow::Borrowed(&node.ident));
        self.push_scope(false, cfg);
        visit::visit_item_mod(self, node);
        self.pop_scope();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let mut cfg = self.visit_cfg_attrs(&node.attrs);

        if self.is_root_scope() && node.sig.ident == "main" {
            // main function in the top-level scope
            self.hints.has_main = true;
        } else {
            for attr in &node.attrs {
                if let syn::Meta::Path(path) = &attr.meta {
                    if path.is_ident("test") {
                        self.hints.has_test = true;
                        cfg.push(LazyConfigFlag::Raw(BExpr::Terminal(BExprAtom::Option {
                            option: "test".to_string(),
                        })));
                    } else if path.is_ident("proc_macro") || path.is_ident("proc_macro_attribute") {
                        self.hints.has_proc_macro = true;
                    }
                }
            }
        }

        self.push_scope(false, cfg);
        visit::visit_item_fn(self, node);
        self.pop_scope();
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.visit_type_attrs(&node.attrs);
        let cfg = self.visit_cfg_attrs(&node.attrs);
        self.push_scope(false, cfg);

        visit::visit_item_struct(self, node);

        self.pop_scope();
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.visit_type_attrs(&node.attrs);
        let cfg = self.visit_cfg_attrs(&node.attrs);
        self.push_scope(false, cfg);

        visit::visit_item_enum(self, node);

        self.pop_scope();
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.visit_type_attrs(&node.attrs);
        let cfg = self.visit_cfg_attrs(&node.attrs);
        self.push_scope(false, cfg);

        visit::visit_item_type(self, node);

        self.pop_scope();
    }
}
