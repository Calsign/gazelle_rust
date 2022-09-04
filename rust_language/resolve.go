package rust_language

import (
	"log"
	"sort"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/repo"
	"github.com/bazelbuild/bazel-gazelle/resolve"
	"github.com/bazelbuild/bazel-gazelle/rule"

	pb "github.com/calsign/gazelle_rust/proto"
)

var builtins = map[string]bool{
	"std":  true,
	"core": true,
}

var provided = map[string]label.Label{
	"runfiles": label.New("rules_rust", "tools/runfiles", "runfiles"),
}

func (l *rustLang) Imports(c *config.Config, r *rule.Rule,
	f *rule.File) []resolve.ImportSpec {

	specs := []resolve.ImportSpec{}

	switch r.Kind() {
	case "rust_library":
		crateName := r.AttrString("crate_name")
		if crateName == "" {
			crateName = r.Name()
		}
		specs = append(specs, resolve.ImportSpec{
			Lang: l.Name(),
			Imp:  crateName,
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
	rc *repo.RemoteCache, r *rule.Rule, imports interface{}, from label.Label) {

	cfg := l.GetConfig(c)

	switch r.Kind() {
	case "rust_library", "rust_binary", "rust_test":
		files := imports.([]*pb.RustImportsResponse)
		deps := map[label.Label]bool{}

		for _, response := range files {
			for _, imp := range response.GetImports() {
				spec := resolve.ImportSpec{
					Lang: l.Name(),
					Imp:  imp,
				}

				var selected label.Label

				if builtins[imp] {
					continue
				} else if override, ok := resolve.FindRuleWithOverride(c, spec, l.Name()); ok {
					selected = override
				} else if _, ok := cfg.LockfileCrates.Crates[spec]; ok {
					var err error
					selected, err = label.Parse(cfg.CratesPrefix + imp)
					if err != nil {
						log.Fatal(err)
					}
				} else if candidates := ix.FindRulesByImportWithConfig(c, spec, l.Name()); len(candidates) >= 1 {
					if len(candidates) == 1 {
						selected = candidates[0].Label
					} else {
						log.Printf("multiple matches found for %s: %v\n", imp, candidates)
						continue
					}
				} else if override, ok := provided[imp]; ok {
					selected = override
				} else {
					log.Printf("no match for %s\n", imp)
					continue
				}

				deps[selected] = true
			}
		}

		r.SetAttr("deps", finalizeDeps(deps, from))
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
