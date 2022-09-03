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
        .map(|ident| ident.to_string())
        .collect();

    Ok(imports)
}

#[derive(Debug)]
struct AstVisitor<'ast> {
    imports: HashSet<&'ast syn::Ident>,
    rename_stack: VecDeque<Vec<&'ast syn::Ident>>,
    scope_renames: HashSet<&'ast syn::Ident>,
}

impl<'ast> Default for AstVisitor<'ast> {
    fn default() -> Self {
        let mut rename_stack = VecDeque::new();
        rename_stack.push_back(Vec::new());
        Self {
            imports: HashSet::default(),
            rename_stack,
            scope_renames: HashSet::default(),
        }
    }
}

impl<'ast> AstVisitor<'ast> {
    fn add_import(&mut self, ident: &'ast syn::Ident) {
        if !self.scope_renames.contains(ident) {
            self.imports.insert(ident);
        }
    }

    fn add_rename(&mut self, rename: &'ast syn::Ident) {
        if !self.scope_renames.contains(rename) {
            self.scope_renames.insert(rename);
            self.rename_stack.back_mut().unwrap().push(rename);
        }
    }

    fn push_scope(&mut self) {
        // TODO: create stack entry lazily so that we avoid it if there are no renames in this scope
        self.rename_stack.push_back(Vec::new());
    }

    fn pop_scope(&mut self) {
        for rename in self.rename_stack.pop_back().expect("hit bottom of stack") {
            self.scope_renames.remove(rename);
        }
    }
}

impl<'ast> Visit<'ast> for AstVisitor<'ast> {
    fn visit_use_name(&mut self, node: &'ast syn::UseName) {
        self.add_import(&node.ident);
    }

    fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
        self.add_import(&node.ident);
    }

    fn visit_use_rename(&mut self, node: &'ast syn::UseRename) {
        self.add_import(&node.ident);
        self.add_rename(&node.rename);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if node.segments.len() > 1 {
            self.add_import(&node.segments[0].ident);
        }
        visit::visit_path(self, node);
    }

    fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate) {
        self.add_import(&node.ident);
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.push_scope();
        visit::visit_block(self, node);
        self.pop_scope();
    }
}
