package rust_language

import (
	"log"
	"sort"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/repo"
	"github.com/bazelbuild/bazel-gazelle/resolve"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

var builtins = map[string]bool{
	"std":        true,
	"core":       true,
	"proc_macro": true,
}

var provided = map[string]label.Label{
	"runfiles": label.New("rules_rust", "tools/runfiles", "runfiles"),
}

func getCrateName(r *rule.Rule) string {
	crateName := r.AttrString("crate_name")
	if crateName == "" {
		crateName = r.Name()
	}
	return crateName
}

func (l *rustLang) Imports(c *config.Config, r *rule.Rule,
	f *rule.File) []resolve.ImportSpec {

	specs := []resolve.ImportSpec{}

	switch r.Kind() {
	case "rust_library":
		specs = append(specs, resolve.ImportSpec{
			Lang: l.Name(),
			Imp:  getCrateName(r),
		})
	case "rust_proc_macro":
		specs = append(specs, resolve.ImportSpec{
			Lang: procMacroLangName,
			Imp:  getCrateName(r),
		})
	case "rust_proto_library", "rust_grpc_library":
		specs = append(specs, resolve.ImportSpec{
			Lang: l.Name(),
			Imp:  r.Name(),
		})
	}

	return specs
}

func (*rustLang) Embeds(r *rule.Rule, from label.Label) []label.Label {
	return nil
}

func (*rustLang) CrossResolve(c *config.Config, ix *resolve.RuleIndex,
	spec resolve.ImportSpec, lang string) []resolve.FindResult {

	return []resolve.FindResult{}
}

func (l *rustLang) Resolve(c *config.Config, ix *resolve.RuleIndex,
	rc *repo.RemoteCache, r *rule.Rule, ruleData interface{}, from label.Label) {

	cfg := l.GetConfig(c)

	if SliceContains(commonDefs, r.Kind()) {
		ruleData := ruleData.(RuleData)
		deps := map[label.Label]bool{}
		procMacroDeps := map[label.Label]bool{}

		var crateName string
		if ruleData.testedCrate != nil {
			// if this is an associated rust_test, the crate name is the one from the tested target
			crateName = getCrateName(ruleData.testedCrate)
		} else {
			crateName = getCrateName(r)
		}

		for _, response := range ruleData.responses {
			var imports []string

			if r.Kind() == "rust_test" {
				if ruleData.testedCrate == nil {
					// this is a standalone test
					imports = append(response.GetImports(), response.GetTestImports()...)
				} else {
					// this is a test associated with another target; don't duplicate the deps
					imports = response.GetTestImports()
				}
			} else {
				imports = response.GetImports()
			}

			for _, imp := range imports {
				if imp == crateName {
					// you are allowed to import yourself
					continue
				}

				is_proc_macro := false

				label, found := l.resolveCrate(cfg, c, ix, l.Name(), imp)
				if label != nil {
					is_proc_macro = false
				}
				if !found {
					label, found = l.resolveCrate(cfg, c, ix, procMacroLangName, imp)
					if label != nil {
						is_proc_macro = true
					}
				}

				if proc_macro, ok := cfg.ProcMacroOverrides[imp]; ok {
					// user-defined override
					// NOTE: well-known overrides are handled in lockfile_crates.go
					is_proc_macro = proc_macro
				}

				if found {
					if label != nil {
						if is_proc_macro {
							procMacroDeps[*label] = true
						} else {
							deps[*label] = true
						}
					}
				} else {
					log.Printf("no match for %s\n", imp)
				}
			}
		}

		maybeSetAttrStrings(r, "deps", finalizeDeps(deps, from))
		maybeSetAttrStrings(r, "proc_macro_deps", finalizeDeps(procMacroDeps, from))
	}
}

func maybeSetAttrStrings(r *rule.Rule, attr string, val []string) {
	if len(val) > 0 {
		r.SetAttr(attr, val)
	} else {
		r.DelAttr(attr)
	}
}

func (l *rustLang) resolveCrate(cfg *rustConfig, c *config.Config, ix *resolve.RuleIndex,
	lang string, imp string) (*label.Label, bool) {
	spec := resolve.ImportSpec{
		Lang: lang,
		Imp:  imp,
	}

	if builtins[spec.Imp] {
		return nil, true
	} else if override, ok := resolve.FindRuleWithOverride(c, spec, l.Name()); ok {
		return &override, true
	} else if crateName, ok := cfg.LockfileCrates.Crates[spec]; ok {
		var err error
		label, err := label.Parse(cfg.CratesPrefix + crateName)
		if err != nil {
			log.Fatal(err)
		}
		return &label, true
	} else if candidates := ix.FindRulesByImportWithConfig(c, spec, l.Name()); len(candidates) >= 1 {
		if len(candidates) == 1 {
			return &candidates[0].Label, true
		} else {
			candidateLabels := []string{}
			for _, candidate := range candidates {
				candidateLabels = append(candidateLabels, candidate.Label.String())
			}
			log.Printf("multiple matches found for %s: [%s]\n", spec.Imp, strings.Join(candidateLabels, ", "))
			return nil, true
		}
	} else if override, ok := provided[spec.Imp]; ok {
		return &override, true
	} else {
		return nil, false
	}
}

func finalizeDeps(deps map[label.Label]bool, from label.Label) []string {
	result := make([]string, 0, len(deps))
	for label := range deps {
		result = append(result, label.Rel(from.Repo, from.Pkg).String())
	}
	sort.Strings(result)
	return result
}
