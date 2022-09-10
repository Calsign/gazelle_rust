package rust_language

import (
	"flag"
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
)

// Available directives
var (
	// Path to Cargo.Bazel.lock.
	// Use either rust_lockfile or rust_cargo_lockfile, not both.
	// Must also specify rust_crates_prefix.
	lockfileDirective string = "rust_lockfile"

	// Path to Cargo.lock.
	cargoLockfileDirective string = "rust_cargo_lockfile"

	// Label prefix for external crates, e.g. @crates//:
	cratesPrefixDirective string = "rust_crates_prefix"

	// Override whether an external crate should be considered a proc_macro crate.
	// usage: # gazelle:rust_override_proc_macro <crate name> <true|false>
	procMacroOverrideDirective string = "rust_override_proc_macro"

	// Remove an external crate from the "unused crates" warning.
	allowUnusedCrateDirective string = "rust_allow_unused_crate"
)

type rustConfig struct {
	LockfileCrates     *LockfileCrates
	CratesPrefix       string
	ProcMacroOverrides map[string]bool
	KindMapInverse     map[string]string
}

type scopedCrateSet struct {
	LockfileCrates      *LockfileCrates
	Pkg                 string
	AllowedUnusedCrates map[string]bool
}

type rustLang struct {
	Parser       *Parser
	AllCrateSets []scopedCrateSet
}

func NewLanguage() language.Language {
	return &rustLang{
		Parser:       NewParser(),
		AllCrateSets: []scopedCrateSet{},
	}
}

func (*rustLang) Name() string { return langName }

var (
	commonDefs []string = []string{"rust_library", "rust_binary", "rust_test",
		"rust_proc_macro", "rust_shared_library", "rust_static_library"}
	protoDefs []string = []string{"rust_proto_library", "rust_grpc_library"}
)

func (*rustLang) Kinds() map[string]rule.KindInfo {
	kinds := make(map[string]rule.KindInfo)

	for _, commonDef := range commonDefs {
		kinds[commonDef] = rule.KindInfo{
			NonEmptyAttrs:  map[string]bool{"srcs": true},
			MergeableAttrs: map[string]bool{"srcs": true, "deps": true, "proc_macro_deps": true},
			ResolveAttrs:   map[string]bool{"deps": true, "proc_macro_deps": true},
		}
	}

	for _, protoDef := range protoDefs {
		kinds[protoDef] = rule.KindInfo{
			MergeableAttrs: map[string]bool{},
			ResolveAttrs:   map[string]bool{},
		}
	}

	return kinds
}

func (*rustLang) Loads() []rule.LoadInfo {
	return []rule.LoadInfo{
		{
			Name:    "@rules_rust//rust:defs.bzl",
			Symbols: commonDefs,
		},
		{
			Name:    "@rules_rust//proto:proto.bzl",
			Symbols: protoDefs,
		},
	}
}

func (*rustLang) Fix(c *config.Config, f *rule.File) {}

func (*rustLang) RegisterFlags(fs *flag.FlagSet, cmd string, c *config.Config) {
}

func (l *rustLang) CheckFlags(fs *flag.FlagSet, c *config.Config) error {
	return nil
}

func (*rustLang) KnownDirectives() []string {
	return []string{lockfileDirective, cargoLockfileDirective, cratesPrefixDirective,
		procMacroOverrideDirective, allowUnusedCrateDirective}
}

func (l *rustLang) GetConfig(c *config.Config) *rustConfig {
	if _, ok := c.Exts[l.Name()]; !ok {
		c.Exts[l.Name()] = &rustConfig{
			LockfileCrates:     EmptyLockfileCrates(),
			CratesPrefix:       "",
			ProcMacroOverrides: make(map[string]bool),
			KindMapInverse:     make(map[string]string),
		}
	}
	return c.Exts[l.Name()].(*rustConfig)
}

func (l *rustLang) Configure(c *config.Config, rel string, f *rule.File) {
	cfg := l.GetConfig(c)

	addCrateSet := func(directive rule.Directive, cargo bool) {
		// Storing the crate set in the configuration allows for multiple instances of
		// crate_universe or vendored crates in the same repo.
		lockfile := path.Join(c.RepoRoot, rel, directive.Value)
		cfg.LockfileCrates = l.NewLockfileCrates(c, lockfile, cargo)

		allowedUnusedCrates := make(map[string]bool)
		if f != nil {
			for _, directive := range f.Directives {
				if directive.Key == allowUnusedCrateDirective {
					allowedUnusedCrates[directive.Value] = true
				}
			}
		}

		// Track all known crate sets.
		l.AllCrateSets = append(l.AllCrateSets, scopedCrateSet{
			LockfileCrates:      cfg.LockfileCrates,
			Pkg:                 rel,
			AllowedUnusedCrates: allowedUnusedCrates,
		})
	}

	if f != nil {
		for _, directive := range f.Directives {
			if directive.Key == lockfileDirective {
				addCrateSet(directive, false)
			} else if directive.Key == cargoLockfileDirective {
				addCrateSet(directive, true)
			} else if directive.Key == cratesPrefixDirective {
				cfg.CratesPrefix = directive.Value
			} else if directive.Key == procMacroOverrideDirective {
				split := strings.Split(directive.Value, " ")
				if len(split) != 2 || (split[1] != "true" && split[1] != "false") {
					l.Log(c, logFatal, f, "bad %s, should be gazelle:%s <crate> <true|false>",
						procMacroOverrideDirective, procMacroOverrideDirective)
				}
				val, err := strconv.ParseBool(split[1])
				if err != nil {
					l.Log(c, logFatal, f, "bad %s, should be gazelle:%s <crate> <true|false>",
						procMacroOverrideDirective, procMacroOverrideDirective)
				}
				cfg.ProcMacroOverrides[split[0]] = val
			}
		}
	}

	for k, v := range c.KindMap {
		cfg.KindMapInverse[v.KindName] = k
	}
}

func (l *rustLang) DoneResolving(c *config.Config) {
	// NOTE: This is part of the gazelle interface as the result of a patch. If the patch is not
	// applied, things will still work, but you will not get support for reporting unused crates.

	for _, crateSet := range l.AllCrateSets {
		unusedCrates := crateSet.LockfileCrates.UnusedCrates(crateSet.AllowedUnusedCrates)
		if len(unusedCrates) > 0 {
			l.Log(c, logWarn, crateSet.Pkg, "unused crates: [%s]", strings.Join(unusedCrates, ", "))
		}
	}
}

func (l *rustLang) GetMappedKindInverse(c *config.Config, kind string) string {
	if mapped, ok := l.GetConfig(c).KindMapInverse[kind]; ok {
		return mapped
	} else {
		return kind
	}
}
