package rust_language

import (
	"flag"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

var langName string = "rust"

type rustLang struct {
	Parser *Parser
}

func NewLanguage() language.Language {
	return &rustLang{Parser: NewParser()}
}

func (*rustLang) Name() string { return langName }

func (*rustLang) Kinds() map[string]rule.KindInfo {
	return map[string]rule.KindInfo{
		"rust_library": {
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true},
			ResolveAttrs:   map[string]bool{"deps": true},
		},
		"rust_binary": {
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true},
			ResolveAttrs:   map[string]bool{"deps": true},
		},
		"rust_test": {
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true},
			ResolveAttrs:   map[string]bool{"deps": true},
		},
		"rust_proto_library": {
			MergeableAttrs: map[string]bool{},
			ResolveAttrs:   map[string]bool{},
		},
		"rust_grpc_library": {
			MergeableAttrs: map[string]bool{},
			ResolveAttrs:   map[string]bool{},
		},
	}
}

func (*rustLang) Loads() []rule.LoadInfo {
	return []rule.LoadInfo{
		{
			Name:    "@rules_rust//rust:defs.bzl",
			Symbols: []string{"rust_library", "rust_binary", "rust_test"},
		},
		{
			Name:    "@rules_rust//proto:proto.bzl",
			Symbols: []string{"rust_proto_library", "rust_grpc_library"},
		},
	}
}

func (*rustLang) Fix(c *config.Config, f *rule.File) {}

func (*rustLang) RegisterFlags(fs *flag.FlagSet, cmd string,
	c *config.Config) {
}

func (*rustLang) CheckFlags(fs *flag.FlagSet, c *config.Config) error {
	return nil
}

func (*rustLang) KnownDirectives() []string {
	return nil
}

func (*rustLang) Configure(c *config.Config, rel string, f *rule.File) {
}
