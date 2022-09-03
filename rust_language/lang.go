package rust_language

import (
	"flag"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

type rustLang struct {
	Parser *Parser
}

func NewLanguage() language.Language {
	return &rustLang{Parser: NewParser()}
}

func (*rustLang) Name() string { return "rust" }

func (*rustLang) Kinds() map[string]rule.KindInfo {
	return map[string]rule.KindInfo{
		"rust_library": {
			MergeableAttrs: map[string]bool{"deps": true},
			ResolveAttrs:   map[string]bool{"deps": true},
		},
	}
}

func (*rustLang) Loads() []rule.LoadInfo {
	return []rule.LoadInfo{
		{
			Name:    "@rules_rust//rust:defs.bzl",
			Symbols: []string{"rust_library"},
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
