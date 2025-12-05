package rust_language

import (
	"sort"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/repo"
	"github.com/bazelbuild/bazel-gazelle/resolve"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

func getCrateName(r *rule.Rule) string {
	crateName := r.AttrString("crate_name")
	if crateName == "" {
		crateName = r.Name()
	}
	return crateName
}

func (l *rustLang) Imports(c *config.Config, r *rule.Rule,
	f *rule.File) []resolve.ImportSpec {

	// return nil by default
	var specs []resolve.ImportSpec

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

	resolvableDefs := make([]string, 0, len(commonDefs)+len(cargoDefs))
	resolvableDefs = append(resolvableDefs, commonDefs...)
	resolvableDefs = append(resolvableDefs, cargoDefs...)

	if SliceContains(resolvableDefs, r.Kind()) {
		ruleData := ruleData.(RuleData)
		deps := map[label.Label]bool{}
		procMacroDeps := map[label.Label]bool{}

		var crateName string
		if ruleData.testedCrate != nil {
			// test crates have to depend on the tested crate to be able to import them directly
			crateName = ""
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
				// TODO(will): not doing this for rust_binary because this fixes the case where a
				// binary uses a library of the same name, which happens for the auto lib.rs and
				// main.rs bins/libs, but unclear if this is correct in all cases
				if crateName != "" && imp == crateName && r.Kind() != "rust_binary" {
					// you are allowed to import yourself
					continue
				}

				is_proc_macro := false

				label, found := l.resolveCrate(cfg, c, ix, l.Name(), imp, from)
				if label != nil {
					is_proc_macro = false
				}
				if !found {
					label, found = l.resolveCrate(cfg, c, ix, procMacroLangName, imp, from)
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
					l.Log(c, logErr, from, "no match for %s\n", imp)
				}
			}
		}

		if ruleData.dependsOnBuildScript {
			build_script_label, err := label.Parse(":build_script")
			if err != nil {
				l.Log(c, logFatal, from, "bad build script label: %v\n", err)
			}
			deps[build_script_label] = true
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
	lang string, imp string, from label.Label) (*label.Label, bool) {
	spec := resolve.ImportSpec{
		Lang: lang,
		Imp:  imp,
	}

	if Builtins[spec.Imp] {
		return nil, true
	} else if override, ok := resolve.FindRuleWithOverride(c, spec, l.Name()); ok {
		return &override, true
	} else if crateName, ok := cfg.LockfileCrates.Crates[spec]; ok {
		var err error
		label, err := label.Parse(cfg.CratesPrefix + crateName)
		if err != nil {
			l.Log(c, logFatal, from, "bad %s: %v\n", cratesPrefixDirective, err)
		}

		// track this crate as used
		cfg.LockfileCrates.UsedCrates[crateName] = true

		return &label, true
	} else if candidates := ix.FindRulesByImportWithConfig(c, spec, l.Name()); len(candidates) >= 1 {
		if len(candidates) == 1 {
			return &candidates[0].Label, true
		} else {
			candidateLabels := []string{}
			for _, candidate := range candidates {
				candidateLabels = append(candidateLabels, candidate.Label.String())
			}
			l.Log(c, logErr, from, "multiple matches found for %s: [%s]\n", spec.Imp, strings.Join(candidateLabels, ", "))
			return nil, true
		}
	} else if override, ok := Provided[lang][spec.Imp]; ok {
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
