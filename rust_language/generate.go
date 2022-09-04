package rust_language

import (
	"log"
	"path"
	"strings"

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

	if args.File != nil {
		for _, rule := range args.File.Rules {
			responses := []*pb.RustImportsResponse{}

			for _, file := range rule.AttrStrings("srcs") {
				filesInExistingRules[file] = true

				if strings.HasSuffix(file, ".rs") {
					response := l.parseFile(file, args)
					responses = append(responses, response)
				}
			}

			result.Gen = append(result.Gen, rule)
			result.Imports = append(result.Imports, responses)
		}
	}

	for _, file := range args.RegularFiles {
		if !filesInExistingRules[file] && strings.HasSuffix(file, ".rs") {
			response := l.parseFile(file, args)

			inferredKind := l.inferRuleKind(file, dirname, response)

			rule := rule.NewRule(inferredKind, strings.TrimSuffix(file, ".rs"))
			rule.SetAttr("srcs", []string{file})

			result.Gen = append(result.Gen, rule)
			result.Imports = append(result.Imports, []*pb.RustImportsResponse{response})
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
