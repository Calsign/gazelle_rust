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

	lockfileDirective          string = "rust_lockfile"
	cargoLockfileDirective     string = "rust_cargo_lockfile"
	cratesPrefixDirective      string = "rust_crates_prefix"
	procMacroOverrideDirective string = "rust_override_proc_macro"
	checkFlag                  string = "rust_check"
)

type rustConfig struct {
	LockfileCrates     *LockfileCrates
	CratesPrefix       string
	ProcMacroOverrides map[string]bool
	KindMapInverse     map[string]string
	Check              bool
}

type scopedCrateSet struct {
	LockfileCrates *LockfileCrates
	Pkg            string
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
	fs.Bool(checkFlag, false, "non-fatal warnings and errors become fatal")
}

func (l *rustLang) CheckFlags(fs *flag.FlagSet, c *config.Config) error {
	cfg := l.GetConfig(c)
	shouldCheck, err := strconv.ParseBool(fs.Lookup(checkFlag).Value.String())
	if err != nil {
		return err
	}
	cfg.Check = shouldCheck

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
		// Track all known crate sets.
		l.AllCrateSets = append(l.AllCrateSets, scopedCrateSet{
			LockfileCrates: cfg.LockfileCrates,
			Pkg:            rel,
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

func (l *rustLang) GetMappedKindInverse(c *config.Config, kind string) string {
	if mapped, ok := l.GetConfig(c).KindMapInverse[kind]; ok {
		return mapped
	} else {
		return kind
	}
}
