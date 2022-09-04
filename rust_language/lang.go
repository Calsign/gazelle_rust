package rust_language

import (
	"flag"
	"log"
	"path"
	"strconv"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

var (
	langName          string = "rust"
	procMacroLangName string = "rust_proc_macro"

	lockfileDirective          string = "rust_lockfile"
	cargoLockfileDirective     string = "rust_cargo_lockfile"
	cratesPrefixDirective      string = "rust_crates_prefix"
	procMacroOverrideDirective string = "rust_override_proc_macro"
)

type rustConfig struct {
	LockfileCrates     *LockfileCrates
	CratesPrefix       string
	ProcMacroOverrides map[string]bool
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
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true, "proc_macro_deps": true},
			ResolveAttrs:   map[string]bool{"deps": true, "proc_macro_deps": true},
		},
		"rust_binary": {
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true, "proc_macro_deps": true},
			ResolveAttrs:   map[string]bool{"deps": true, "proc_macro_deps": true},
		},
		"rust_test": {
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true, "proc_macro_deps": true},
			ResolveAttrs:   map[string]bool{"deps": true, "proc_macro_deps": true},
		},
		"rust_proc_macro": {
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true, "proc_macro_deps": true},
			ResolveAttrs:   map[string]bool{"deps": true, "proc_macro_deps": true},
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
			Symbols: []string{"rust_library", "rust_binary", "rust_test", "rust_proc_macro"},
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
	return []string{lockfileDirective, cargoLockfileDirective,
		cratesPrefixDirective, procMacroOverrideDirective}
}

func (l *rustLang) GetConfig(c *config.Config) *rustConfig {
	if _, ok := c.Exts[l.Name()]; !ok {
		c.Exts[l.Name()] = &rustConfig{
			LockfileCrates:     EmptyLockfileCrates(),
			CratesPrefix:       "",
			ProcMacroOverrides: make(map[string]bool),
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
			} else if directive.Key == procMacroOverrideDirective {
				split := strings.Split(directive.Value, " ")
				if len(split) != 2 || (split[1] != "true" && split[1] != "false") {
					log.Fatalf("%s: bad %s, should be gazelle:%s <crate> <true|false>",
						f.Path, procMacroOverrideDirective, procMacroOverrideDirective)
				}
				val, err := strconv.ParseBool(split[1])
				if err != nil {
					log.Fatal(err)
				}
				cfg.ProcMacroOverrides[split[0]] = val
			}
		}
	}
}
