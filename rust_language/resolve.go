package rust_language

import (
	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/repo"
	"github.com/bazelbuild/bazel-gazelle/resolve"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

func (*rustLang) Imports(c *config.Config, r *rule.Rule,
	f *rule.File) []resolve.ImportSpec {
	specs := []resolve.ImportSpec{}

	return specs
}

func (*rustLang) Embeds(r *rule.Rule, from label.Label) []label.Label {
	return nil
}

func (*rustLang) CrossResolve(c *config.Config, ix *resolve.RuleIndex,
	spec resolve.ImportSpec, lang string) []resolve.FindResult {
	return []resolve.FindResult{}
}

func (*rustLang) Resolve(c *config.Config, ix *resolve.RuleIndex,
	rc *repo.RemoteCache, r *rule.Rule, imports interface{}, from label.Label) {

}
