package rust_language

import (
	"flag"
	"path"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

var (
	langName               string = "rust"
	lockfileDirective      string = "rust_lockfile"
	cargoLockfileDirective string = "rust_cargo_lockfile"
	cratesPrefixDirective  string = "rust_crates_prefix"
)

type rustConfig struct {
	LockfileCrates *LockfileCrates
	CratesPrefix   string
}

type rustLang struct {
	Parser *Parser
}

func NewLanguage() language.Language {
	return &rustLang{
		Parser: NewParser(),
	}
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
	return []string{lockfileDirective, cargoLockfileDirective, cratesPrefixDirective}
}

func (l *rustLang) GetConfig(c *config.Config) *rustConfig {
	if _, ok := c.Exts[l.Name()]; !ok {
		c.Exts[l.Name()] = &rustConfig{
			LockfileCrates: EmptyLockfileCrates(),
		}
	}
	return c.Exts[l.Name()].(*rustConfig)
}

func (l *rustLang) Configure(c *config.Config, rel string, f *rule.File) {
	cfg := l.GetConfig(c)

	if f != nil {
		for _, directive := range f.Directives {
			if directive.Key == lockfileDirective {
				lockfile := path.Join(c.RepoRoot, rel, directive.Value)
				cfg.LockfileCrates = NewLockfileCrates(l.Parser, lockfile, false)
			} else if directive.Key == cargoLockfileDirective {
				lockfile := path.Join(c.RepoRoot, rel, directive.Value)
				cfg.LockfileCrates = NewLockfileCrates(l.Parser, lockfile, true)
			} else if directive.Key == cratesPrefixDirective {
				cfg.CratesPrefix = directive.Value
			}
		}
	}
}
