package rust_language

import (
	"fmt"
	"log"
	"sort"
	"strings"

	bzl "github.com/bazelbuild/buildtools/build"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/repo"
	"github.com/bazelbuild/bazel-gazelle/resolve"
	"github.com/bazelbuild/bazel-gazelle/rule"

	pb "github.com/calsign/gazelle_rust/proto"
)

var defaultCfg = "//conditions:default"

type CfgList struct {
	// map from dep to cfg expression under which it is valid
	deps map[string]bzl.Expr
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

	if SliceContains(commonDefs, r.Kind()) {
		ruleData := ruleData.(RuleData)
		deps := CfgList{deps: make(map[string]bzl.Expr)}
		procMacroDeps := CfgList{deps: make(map[string]bzl.Expr)}

		var crateName string
		if ruleData.testedCrate != nil {
			// test crates have to depend on the tested crate to be able to import them directly
			crateName = ""
		} else {
			crateName = getCrateName(r)
		}

		enabledFeatures := make(map[string]bool)
		attrFeatures := r.AttrStrings("crate_features")
		if attrFeatures != nil {
			for _, feature := range attrFeatures {
				enabledFeatures[feature] = true
			}
		}

		for _, response := range ruleData.responses {
			includeTestImports := false
			includeNonTestImports := false

			if r.Kind() == "rust_test" {
				if ruleData.testedCrate == nil {
					// this is a standalone test
					includeTestImports = true
					includeNonTestImports = true
				} else {
					// this is a test associated with another target; don't duplicate the deps
					includeTestImports = true
				}
			} else {
				includeNonTestImports = true
			}

			imports := response.GetImports()

			for _, imp := range imports {
				// TODO(will): not doing this for rust_binary because this fixes the case where a
				// binary uses a library of the same name, which happens for the auto lib.rs and
				// main.rs bins/libs, but unclear if this is correct in all cases
				if crateName != "" && imp.Imp == crateName && r.Kind() != "rust_binary" {
					// you are allowed to import yourself
					continue
				}

				is_proc_macro := false

				label, found := l.resolveCrate(cfg, c, ix, l.Name(), imp.Imp, from)
				if label != nil {
					is_proc_macro = false
				}
				if !found {
					label, found = l.resolveCrate(cfg, c, ix, procMacroLangName, imp.Imp, from)
					if label != nil {
						is_proc_macro = true
					}
				}

				if proc_macro, ok := cfg.ProcMacroOverrides[imp.Imp]; ok {
					// user-defined override
					// NOTE: well-known overrides are handled in lockfile_crates.go
					is_proc_macro = proc_macro
				}

				if found {
					if label != nil {
						if includeNonTestImports {
							// TODO: don't duplicate non-test dependencies for tests
						}

						initialCfgExpr := l.transformCfg(cfg, c, imp.Cfg, from,
							includeTestImports, enabledFeatures)
						simplified, err := l.Parser.SimplifyBExpr(&pb.SimplifyBExprRequest{
							Bexpr: initialCfgExpr})
						if err != nil {
							l.Log(c, logFatal, from, "failed to simplify cfg expr")
						}
						cfgExpr := l.mapRustCfg(cfg, c, simplified.Bexpr, from)

						rel := relLabel(label, from)

						if cfgExpr != nil {
							if is_proc_macro {
								procMacroDeps.deps[rel] = cfgExpr
							} else {
								deps.deps[rel] = cfgExpr
							}
						}
					}
				} else {
					l.Log(c, logErr, from, "no match for %s\n", imp.Imp)
				}
			}
		}

		maybeSetAttrCfgList(r, "deps", deps)
		maybeSetAttrCfgList(r, "proc_macro_deps", procMacroDeps)
	}
}

func maybeSetAttrCfgList(r *rule.Rule, attr string, val CfgList) {
	if len(val.deps) > 0 {
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
			l.Log(c, logErr, from, "multiple matches found for %s: [%s]\n", spec.Imp,
				strings.Join(candidateLabels, ", "))
			return nil, true
		}
	} else if override, ok := Provided[lang][spec.Imp]; ok {
		return &override, true
	} else {
		return nil, false
	}
}

func relLabel(label *label.Label, from label.Label) string {
	return label.Rel(from.Repo, from.Pkg).String()
}

func (cfgList CfgList) BzlExpr() bzl.Expr {
	alwaysDeps := []bzl.Expr{}
	selects := []bzl.Expr{}

	// TODO: sort by dict keys, not by values
	sortedDeps := make([]string, 0, len(cfgList.deps))
	for dep, _ := range cfgList.deps {
		sortedDeps = append(sortedDeps, dep)
	}
	sort.Strings(sortedDeps)

	for _, dep := range sortedDeps {
		cfg := cfgList.deps[dep]
		if s, ok := cfg.(*bzl.StringExpr); ok && s.Value == defaultCfg {
			alwaysDeps = append(alwaysDeps, &bzl.StringExpr{Value: dep})
		} else {
			selects = append(selects, &bzl.CallExpr{
				X: &bzl.Ident{Name: "select"},
				List: []bzl.Expr{&bzl.DictExpr{List: []*bzl.KeyValueExpr{
					&bzl.KeyValueExpr{
						Key: cfg,
						Value: &bzl.ListExpr{List: []bzl.Expr{
							&bzl.StringExpr{Value: dep},
						}},
					},
					&bzl.KeyValueExpr{
						Key:   &bzl.StringExpr{Value: defaultCfg},
						Value: &bzl.ListExpr{},
					},
				}}},
			})
		}
	}

	// only create a starting list if there are things to put in it
	var first bzl.Expr
	if len(alwaysDeps) > 0 {
		first = &bzl.ListExpr{List: alwaysDeps}
	} else {
		first = nil
	}

	return JoinPlus(first, selects)
}

func (cfgList CfgList) add(other CfgList) {
	for dep, cfg := range other.deps {
		if _, ok := other.deps[dep]; ok {
			// TODO: fix
			log.Panicf("got same dep in both sides: %v", dep)
		}
		cfgList.deps[dep] = cfg
	}
}

func exprToCfgList(expr bzl.Expr) (*CfgList, error) {
	switch e := expr.(type) {
	case *bzl.BinaryExpr:
		if e.Op == "+" {
			cfgList := CfgList{deps: make(map[string]bzl.Expr)}
			left, err := exprToCfgList(e.X)
			if err != nil {
				return nil, err
			}
			cfgList.add(*left)
			right, err := exprToCfgList(e.Y)
			if err != nil {
				return nil, err
			}
			cfgList.add(*right)
			return &cfgList, nil
		}
	case *bzl.CallExpr:
		if ident, ok := e.X.(*bzl.Ident); ok && ident.Name == "select" && len(e.List) == 1 {
			if dict, ok := e.List[0].(*bzl.DictExpr); ok {
				cfgList := CfgList{deps: make(map[string]bzl.Expr)}
				for _, keyVal := range dict.List {
					inner, err := exprToCfgList(keyVal.Value)
					if err != nil {
						return nil, err
					}
					// update configuration for all inner deps
					for dep, _ := range inner.deps {
						inner.deps[dep] = keyVal.Key
					}
					cfgList.add(*inner)
				}
				return &cfgList, nil
			}
		}
	case *bzl.ListExpr:
		cfgList := CfgList{deps: make(map[string]bzl.Expr)}
		for _, item := range e.List {
			start, _ := item.Span()
			if str, ok := item.(*bzl.StringExpr); ok {
				if _, ok := cfgList.deps[str.Value]; ok {
					return nil, fmt.Errorf("duplicate depedency on line %d", start.Line)
				}
				cfgList.deps[str.Value] = &bzl.StringExpr{Value: defaultCfg}
			} else {
				return nil, fmt.Errorf("expected string on line %d", start.Line)
			}
		}
		return &cfgList, nil
	}

	start, _ := expr.Span()
	return nil, fmt.Errorf("invalid cfg list expr on line %d", start.Line)
}

func (cfgList CfgList) Merge(other bzl.Expr) bzl.Expr {
	if other != nil {
		existing, err := exprToCfgList(other)
		if err != nil {
			log.Fatalf("failed to merge: %v", err)
			return nil
		}
		cfgList.add(*existing)
	}

	return cfgList.BzlExpr()
}

func JoinPlus(first bzl.Expr, exprs []bzl.Expr) bzl.Expr {
	var result bzl.Expr
	if first != nil {
		result = first
	} else if len(exprs) > 1 {
		result = exprs[0]
		exprs = exprs[1:]
	} else {
		return &bzl.ListExpr{}
	}
	for _, e := range exprs {
		result = &bzl.BinaryExpr{
			Op: "+",
			X:  result,
			Y:  e,
		}
	}
	return result
}

func (l *rustLang) transformCfg(cfg *rustConfig, c *config.Config, bexpr *pb.BExpr,
	from label.Label, isTest bool, features map[string]bool) *pb.BExpr {

	switch e := bexpr.Expr.(type) {
	case *pb.BExpr_Atom:
		var key string

		switch a := e.Atom.Atom.(type) {
		case *pb.BExprAtom_Value:
			// special handling for tests - replace with constant
			if a.Value == "test" {
				return &pb.BExpr{Expr: &pb.BExpr_Constant{Constant: isTest}}
			}

			key = a.Value
		case *pb.BExprAtom_KeyValue:
			// special handling for features - evaluate based on current enabled
			// features and replace with constants
			if a.KeyValue.Key == "feature" {
				if _, ok := features[a.KeyValue.Value]; ok {
					return &pb.BExpr{Expr: &pb.BExpr_Constant{Constant: true}}
				} else {
					return &pb.BExpr{Expr: &pb.BExpr_Constant{Constant: false}}
				}
			}

			key = fmt.Sprintf("%s=%s", a.KeyValue.Key, a.KeyValue.Value)
		}

		if mapped, ok := cfg.CfgMapping[key]; ok {
			return &pb.BExpr{Expr: &pb.BExpr_Atom{
				Atom: &pb.BExprAtom{Atom: &pb.BExprAtom_Value{Value: mapped}},
			}}
		} else {
			// TODO: probably just remove this warning?
			l.Log(c, logWarn, from, "could not find cfg: %s", key)
			return &pb.BExpr{Expr: &pb.BExpr_Constant{Constant: true}}
		}
	case *pb.BExpr_Constant:
		return bexpr
	case *pb.BExpr_Not:
		return &pb.BExpr{Expr: &pb.BExpr_Not{
			Not: l.transformCfg(cfg, c, e.Not, from, isTest, features),
		}}
	case *pb.BExpr_And:
		values := make([]*pb.BExpr, 0, len(e.And.Values))
		for _, v := range e.And.Values {
			values = append(values, v)
		}

		return &pb.BExpr{Expr: &pb.BExpr_And{
			And: &pb.BExprSeq{Values: values},
		}}
	case *pb.BExpr_Or:
		values := make([]*pb.BExpr, 0, len(e.Or.Values))
		for _, v := range e.Or.Values {
			values = append(values, v)
		}

		return &pb.BExpr{Expr: &pb.BExpr_Or{
			Or: &pb.BExprSeq{Values: values},
		}}
	default:
		l.Log(c, logFatal, from, "got invalid bexpr")
		return nil
	}
}

func (l *rustLang) mapRustCfg(cfg *rustConfig, c *config.Config, bexpr *pb.BExpr,
	from label.Label) bzl.Expr {
	switch e := bexpr.Expr.(type) {
	case *pb.BExpr_Atom:
		switch a := e.Atom.Atom.(type) {
		case *pb.BExprAtom_Value:
			return &bzl.StringExpr{Value: a.Value}
		case *pb.BExprAtom_KeyValue:
			l.Log(c, logFatal, from, "got impossible bexpr")
		}
	case *pb.BExpr_Constant:
		if e.Constant {
			return &bzl.StringExpr{Value: defaultCfg}
		} else {
			return nil
		}
	case *pb.BExpr_Not:
		// todo
		return nil
	case *pb.BExpr_And:
		// todo
		return nil
	case *pb.BExpr_Or:
		// todo
		return nil
	}

	l.Log(c, logFatal, from, "got invalid bexpr")
	return nil
}
