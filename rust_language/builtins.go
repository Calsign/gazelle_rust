package rust_language

import (
	"github.com/bazelbuild/bazel-gazelle/label"
)

// modules provided by rust
var Builtins = map[string]bool{
	// standard library
	"std":  true,
	"core": true,

	// proc macros
	"proc_macro": true,

	// primitive types
	// https://doc.rust-lang.org/std/primitive/index.html
	"bool": true,
	"char": true,
	"str":  true,

	"f32": true,
	"f64": true,

	"i8":   true,
	"i16":  true,
	"i32":  true,
	"i64":  true,
	"i128": true,

	"u8":   true,
	"u16":  true,
	"u32":  true,
	"u64":  true,
	"u128": true,

	"isize": true,
	"usize": true,
}

// crates provided by rules_rust
var Provided = map[string]map[string]label.Label{
	langName: {
		"runfiles": label.New("rules_rust", "tools/runfiles", "runfiles"),
	},
	procMacroLangName: {
		"gazelle": label.New("gazelle_rust", "macro", "macro"),
	},
}
