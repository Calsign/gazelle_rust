package rust_language

import (
	"log"
	"path"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"

	pb "github.com/calsign/gazelle_rust/proto"
)

func (l *rustLang) isTestDir(dirname *string) bool {
	return dirname != nil && (*dirname == "test" || *dirname == "tests")
}

func (l *rustLang) isTestFilename(filename string) bool {
	return strings.HasSuffix(filename, "_test.rs") || strings.HasPrefix(filename, "test_")
}

// Infer the default kind for a new target (e.g. rust_library, rust_binary).
func (l *rustLang) inferRuleKind(filename string, dirname *string,
	response *pb.RustImportsResponse) string {

	if response.Hints.HasProcMacro {
		// only proc-macro crates are allowed to have #[proc_macro] functions
		return "rust_proc_macro"
	} else if response.Hints.HasMain {
		// while not necessarily true, having a top-level main function is a strong
		// indicator that this is a binary
		return "rust_binary"
	} else if filename == "main.rs" {
		return "rust_binary"
	} else if filename == "lib.rs" {
		return "rust_library"
	} else if response.Hints.HasTest && (l.isTestDir(dirname) || l.isTestFilename(filename)) {
		// assume that sources with tests in a test/tests directory are integration tests
		// assume that sources with tests with test-like names are integration tests
		return "rust_test"
	} else {
		return "rust_library"
	}
}

type RuleData struct {
	rule      *rule.Rule
	responses []*pb.RustImportsResponse
	// if a test crate referring to another crate, that crate; otherwise, nil
	testedCrate *rule.Rule
}

func getTestCrate(rule *rule.Rule, repo string, pkg string) string {
	crateName := rule.AttrString("crate")
	if crateName != "" {
		label, err := label.Parse(crateName)
		if err == nil {
			rel := label.Rel(repo, pkg)
			if rel.Relative {
				return label.Name
			}
		}
	}
	return ""
}

func (l *rustLang) GenerateRules(args language.GenerateArgs) language.GenerateResult {
	result := language.GenerateResult{}

	filesInExistingRules := map[string]bool{}

	var dirname *string
	if args.Rel == "" {
		dirname = nil
	} else {
		base := path.Base(args.Rel)
		dirname = &base
	}

	// list of all non-rust_test rules; these may generate additional crate test targets
	nonTestRules := []RuleData{}
	// map of crate test rules; key is the non-rust_test rule name that each one refers to
	testRules := make(map[string]*rule.Rule)

	addRule := func(rule *rule.Rule, responses []*pb.RustImportsResponse) {
		ruleData := RuleData{
			rule:        rule,
			responses:   responses,
			testedCrate: nil,
		}

		result.Gen = append(result.Gen, rule)
		result.Imports = append(result.Imports, ruleData)

		if rule.Kind() == "rust_test" {
			if crateName := getTestCrate(rule, args.Config.RepoName, args.Rel); crateName != "" {
				if _, ok := testRules[crateName]; ok {
					log.Printf("%s: found multiple crate test rules for %s\n", args.File.Path, crateName)
				}
				testRules[crateName] = rule
			}
		} else {
			nonTestRules = append(nonTestRules, ruleData)
		}
	}

	if args.File != nil {
		for _, rule := range args.File.Rules {
			// NOTE: Gazelle expects us to create rules using the un-mapped kinds. Since we are
			// re-creating an existing rule, the associated kind is the mapped one, and we need to
			// reset it. It is probably a bug that Gazelle does not already handle this for us.
			rule.SetKind(l.GetMappedKindInverse(args.Config, rule.Kind()))

			if SliceContains(commonDefs, rule.Kind()) {
				responses := []*pb.RustImportsResponse{}

				for _, file := range rule.AttrStrings("srcs") {
					filesInExistingRules[file] = true

					if strings.HasSuffix(file, ".rs") {
						response := l.parseFile(file, args)
						responses = append(responses, response)
					}
				}

				addRule(rule, responses)
			}
		}
	}

	for _, file := range args.RegularFiles {
		if !filesInExistingRules[file] && strings.HasSuffix(file, ".rs") {
			response := l.parseFile(file, args)

			inferredKind := l.inferRuleKind(file, dirname, response)

			rule := rule.NewRule(inferredKind, strings.TrimSuffix(file, ".rs"))
			rule.SetAttr("srcs", []string{file})

			responses := []*pb.RustImportsResponse{response}

			addRule(rule, responses)
		}
	}

	for _, ruleData := range nonTestRules {
		hasTest := false
		for _, response := range ruleData.responses {
			if response.Hints.HasTest {
				hasTest = true
			}
		}

		testRule := testRules[ruleData.rule.Name()]

		if hasTest {
			// create a corresponding test crate target
			if testRule == nil {
				testRule = rule.NewRule("rust_test", ruleData.rule.Name()+"_test")
				testRule.SetAttr("crate", ":"+ruleData.rule.Name())
			}

			result.Gen = append(result.Gen, testRule)
			result.Imports = append(result.Imports, RuleData{
				rule:        testRule,
				responses:   ruleData.responses,
				testedCrate: ruleData.rule,
			})
		} else {
			// TODO: remove test target if we no longer have any tests
		}
	}

	return result
}

func (l *rustLang) parseFile(file string, args language.GenerateArgs) *pb.RustImportsResponse {
	request := &pb.RustImportsRequest{FilePath: path.Join(args.Dir, file)}
	response, err := l.Parser.Parse(request)
	if err != nil {
		log.Fatal(err)
	}
	return response
}
