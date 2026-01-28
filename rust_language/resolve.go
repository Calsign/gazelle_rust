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
	case "rust_prost_library":
		// rules_rust_prost derives the crate name from the proto_library target name
		protoAttr := r.AttrString("proto")
		if protoAttr != "" {
			protoLabel, err := label.Parse(protoAttr)
			if err == nil {
				crateName := strings.ReplaceAll(protoLabel.Name, "-", "_")
				specs = append(specs, resolve.ImportSpec{
					Lang: l.Name(),
					Imp:  crateName,
				})
			}
		}
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

	if SliceContains(resolvableDefs, r.Kind()) {
		ruleData := ruleData.(RuleData)
		deps := map[label.Label]bool{}
		procMacroDeps := map[label.Label]bool{}
		// aliases map: label -> local_name (for renamed dependencies)
		aliases := map[label.Label]string{}

		// Build reverse lookup: local_name -> package_name
		localToPackage := make(map[string]string)
		for packageName, localName := range ruleData.aliases {
			localToPackage[localName] = packageName
		}

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

				// Check if this import is an alias (local_name -> package_name)
				actualCrate := imp
				isAlias := false
				if packageName, ok := localToPackage[imp]; ok {
					actualCrate = packageName
					isAlias = true
				}

				is_proc_macro := false

				resolvedLabel, found := l.resolveCrate(cfg, c, ix, l.Name(), actualCrate, ruleData.parentCrateName, from)
				if resolvedLabel != nil {
					is_proc_macro = false
				}
				if !found {
					resolvedLabel, found = l.resolveCrate(cfg, c, ix, procMacroLangName, actualCrate, ruleData.parentCrateName, from)
					if resolvedLabel != nil {
						is_proc_macro = true
					}
				}

				if proc_macro, ok := cfg.ProcMacroOverrides[actualCrate]; ok {
					// user-defined override
					// NOTE: well-known overrides are handled in lockfile_crates.go
					is_proc_macro = proc_macro
				}

				if found {
					if resolvedLabel != nil {
						if is_proc_macro {
							procMacroDeps[*resolvedLabel] = true
						} else {
							deps[*resolvedLabel] = true
						}
						// If this was an aliased import, record the alias mapping
						if isAlias {
							aliases[*resolvedLabel] = imp
						}
					}
				} else {
					l.Log(c, logErr, from, "no match for %s\n", imp)
				}
			}
		}

		if ruleData.buildScript != nil {
			deps[*ruleData.buildScript] = true
		}

		maybeSetAttrStrings(r, "deps", finalizeDeps(deps, from))
		maybeSetAttrStrings(r, "proc_macro_deps", finalizeDeps(procMacroDeps, from))
		maybeSetAliases(r, aliases, from)
	}
}

func maybeSetAttrStrings(r *rule.Rule, attr string, val []string) {
	if len(val) > 0 {
		r.SetAttr(attr, val)
	} else {
		r.DelAttr(attr)
	}
}

// maybeSetAliases sets the aliases attribute on a rule if there are any aliases.
// The aliases attribute maps dependency labels to their local names (aliases).
func maybeSetAliases(r *rule.Rule, aliases map[label.Label]string, from label.Label) {
	if len(aliases) == 0 {
		r.DelAttr("aliases")
		return
	}

	// Convert to map[string]string with relative labels
	aliasMap := make(map[string]string)
	for lbl, localName := range aliases {
		relLabel := lbl.Rel(from.Repo, from.Pkg).String()
		aliasMap[relLabel] = localName
	}

	r.SetAttr("aliases", aliasMap)
}

func (l *rustLang) resolveCrateVersion(cfg *rustConfig, c *config.Config,
	parentCrateName string, crateToResolve string, from label.Label) string {
	if parentCrateName == "" {
		l.Log(c, logFatal, from, "unable to infer the parent crate name while resolving the version of %s", crateToResolve)
	}

	var dependencyVersions map[string]string = cfg.LockfileCrates.DependenciesPerCrate[parentCrateName]

	version, ok := dependencyVersions[crateToResolve]
	if !ok {
		l.Log(c, logFatal, from, "failed to resolve the version of %s based on the Cargo lockfile", crateToResolve)
	}

	return version
}

func (l *rustLang) resolveCrate(cfg *rustConfig, c *config.Config, ix *resolve.RuleIndex,
	lang string, imp string, parentCrateName string, from label.Label) (*label.Label, bool) {
	spec := resolve.ImportSpec{
		Lang: lang,
		Imp:  imp,
	}

	if Builtins[spec.Imp] {
		return nil, true
	} else if override, ok := resolve.FindRuleWithOverride(c, spec, l.Name()); ok {
		return &override, true
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
	} else if crate, ok := cfg.LockfileCrates.Crates[spec]; ok {
		var crateLabel label.Label
		var err error
		if cfg.LockfileCrates.Multiversion[crate] {
			version := l.resolveCrateVersion(cfg, c, parentCrateName, crate, from)
			crateLabel, err = label.Parse(cfg.CratesPrefix + crate + "-" + version)
		} else {
			crateLabel, err = label.Parse(cfg.CratesPrefix + crate)
		}

		if err != nil {
			l.Log(c, logFatal, from, "bad %s: %v\n", cratesPrefixDirective, err)
		}

		// track this crate as used
		cfg.LockfileCrates.UsedCrates[crate] = true

		return &crateLabel, true
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
